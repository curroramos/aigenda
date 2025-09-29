use crate::agent::ToolRegistry;
use crate::agent::memory::{ConversationMemory, ToolCall, ToolResult, MessageRole};
use crate::error::AppResult;
use serde_json::Value;
use chrono::Utc;
use std::time::Instant;

#[cfg(feature = "ai")]
use uuid::Uuid;

pub struct Agent {
    registry: ToolRegistry,
    claude_client: Option<crate::ai::claude::ClaudeClient>,
    memory: ConversationMemory,
    session_id: String,
}

impl Agent {
    pub fn new() -> AppResult<Self> {
        let mut registry = ToolRegistry::new();

        // Auto-discover and register all tools
        registry.auto_discover_tools()?;

        Ok(Self {
            registry,
            claude_client: None,
            memory: ConversationMemory::new(50, 8000), // 50 messages, ~8K tokens
            session_id: Uuid::new_v4().to_string(),
        })
    }

    pub fn new_with_memory_limits(max_messages: usize, max_tokens: usize) -> AppResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.auto_discover_tools()?;

        Ok(Self {
            registry,
            claude_client: None,
            memory: ConversationMemory::new(max_messages, max_tokens),
            session_id: Uuid::new_v4().to_string(),
        })
    }

    pub fn with_claude_client(mut self, client: crate::ai::claude::ClaudeClient) -> Self {
        self.claude_client = Some(client);
        self
    }

    pub async fn execute_command(&mut self, input: &str) -> AppResult<String> {
        // Store user message in memory
        self.memory.add_user_message(input.to_string());

        // Generate dynamic prompt with available tools and conversation history
        let prompt = self.generate_prompt(input).await?;

        // Send to Claude API
        if let Some(client) = &self.claude_client {
            let response = client.chat(&prompt).await?;
            let result = self.process_response(&response).await?;

            // Store assistant response in memory
            self.memory.add_assistant_message(response, None); // Tool calls will be added during processing

            Ok(result)
        } else {
            Err(crate::error::AppError::Storage("Claude client not configured".to_string()))
        }
    }

    async fn generate_prompt(&self, user_input: &str) -> AppResult<String> {
        let tools_description = self.registry.generate_enhanced_tools_description();
        let conversation_context = self.memory.get_context_for_prompt(true);
        let recent_tools = self.memory.get_recent_tool_usage();

        let recent_tools_hint = if !recent_tools.is_empty() {
            format!("Recently used tools: {}\n", recent_tools.join(", "))
        } else {
            String::new()
        };

        let prompt = format!(
            r#"You are an AI assistant with access to various tools for managing notes and other tasks.

{}Available Tools:
{}

{}Current User Request: {}

Please analyze the user's request and determine which tools to use. For each tool call, respond with JSON in this format:
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        // tool-specific parameters
    }}
}}

If multiple tool calls are needed, provide them as a JSON array.

Important:
- Consider the conversation history when interpreting the user's request
- Use the most appropriate tool and action based on the detailed schemas provided
- Validate that required parameters are included
- If the request is ambiguous, prefer the most recent context from conversation history
"#,
            conversation_context, tools_description, recent_tools_hint, user_input
        );

        Ok(prompt)
    }

    async fn process_response(&mut self, response: &str) -> AppResult<String> {
        // Try to extract JSON from Claude's response
        if let Some(json_str) = self.extract_json_from_response(response) {
            // Try parsing as single tool call
            if let Ok(tool_call) = serde_json::from_str::<Value>(&json_str) {
                return self.execute_tool_call(&tool_call).await;
            }
            // Try parsing as array of tool calls
            if let Ok(tool_calls) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let mut results = Vec::new();
                for call in tool_calls {
                    let result = self.execute_tool_call(&call).await?;
                    results.push(result);
                }
                return Ok(results.join("\n"));
            }
        }

        // If no JSON found or parsing failed, return Claude's response as-is
        Ok(response.to_string())
    }

    fn extract_json_from_response(&self, response: &str) -> Option<String> {
        // Look for JSON objects in the response
        let mut brace_count = 0;
        let mut start_idx = None;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in response.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' if !in_string => {
                    if brace_count == 0 {
                        start_idx = Some(i);
                    }
                    brace_count += 1;
                }
                '}' if !in_string => {
                    brace_count -= 1;
                    if brace_count == 0 && start_idx.is_some() {
                        let start = start_idx.unwrap();
                        let json_str = &response[start..=i];
                        return Some(json_str.to_string());
                    }
                }
                _ => {}
            }
        }

        None
    }

    async fn execute_tool_call(&mut self, call: &Value) -> AppResult<String> {
        let tool_name = call["tool"].as_str().ok_or_else(||
            crate::error::AppError::Storage("Missing tool name".to_string()))?;

        let action = call["action"].as_str().ok_or_else(||
            crate::error::AppError::Storage("Missing action".to_string()))?;

        let parameters = &call["parameters"];

        if let Some(tool) = self.registry.get_tool(tool_name) {
            let start_time = Instant::now();
            let call_id = Uuid::new_v4().to_string();

            // Create tool call record
            let tool_call = ToolCall {
                id: call_id.clone(),
                tool_name: tool_name.to_string(),
                action: action.to_string(),
                parameters: parameters.clone(),
                timestamp: Utc::now(),
            };

            // Execute the tool
            let result = tool.execute(action, parameters).await;
            let execution_time = start_time.elapsed().as_millis() as u64;

            // Create tool result record
            let tool_result = ToolResult {
                call_id: call_id.clone(),
                tool_name: tool_name.to_string(),
                action: action.to_string(),
                result: match &result {
                    Ok(r) => r.clone(),
                    Err(e) => format!("Error: {}", e),
                },
                success: result.is_ok(),
                timestamp: Utc::now(),
                execution_time_ms: execution_time,
            };

            // Store the tool execution in memory
            self.memory.add_tool_results(vec![tool_result]);

            result
        } else {
            Err(crate::error::AppError::Storage(format!("Unknown tool: {}", tool_name)))
        }
    }

    pub fn list_available_tools(&self) -> Vec<String> {
        self.registry.list_tools()
    }

    // Memory management methods
    pub fn get_conversation_history(&self) -> String {
        self.memory.get_context_for_prompt(false)
    }

    pub fn clear_memory(&mut self) {
        self.memory.clear();
    }

    pub fn get_memory_stats(&self) -> (usize, usize) {
        (self.memory.message_count(), self.memory.context_token_count())
    }

    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    // Tool schema inspection
    pub fn get_tool_schemas(&self) -> Vec<String> {
        self.registry.get_enhanced_schemas()
    }
}