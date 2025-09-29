use crate::agent::ToolRegistry;
use crate::agent::memory::{ConversationMemory, ToolCall, ToolResult, MessageRole};
use crate::error::AppResult;
use serde_json::Value;
use chrono::Utc;
use std::time::Instant;
use std::io::{self, Write};

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

        // Load persistent memory
        let memory_path = ConversationMemory::get_memory_file_path();
        let memory = ConversationMemory::load_from_file(&memory_path, 50, 8000)?;

        Ok(Self {
            registry,
            claude_client: None,
            memory,
            session_id: Uuid::new_v4().to_string(),
        })
    }

    pub fn new_with_memory_limits(max_messages: usize, max_tokens: usize) -> AppResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.auto_discover_tools()?;

        // Load persistent memory with custom limits
        let memory_path = ConversationMemory::get_memory_file_path();
        let memory = ConversationMemory::load_from_file(&memory_path, max_messages, max_tokens)?;

        Ok(Self {
            registry,
            claude_client: None,
            memory,
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

        // Start the agent execution chain
        let mut full_conversation = String::new();
        let mut execution_context = String::new();
        let mut continue_loop = true;
        let mut loop_count = 0;
        let max_loops = 5; // Prevent infinite loops

        while continue_loop && loop_count < max_loops {
            loop_count += 1;

            // Generate dynamic prompt based on current context
            let prompt = if loop_count == 1 {
                self.generate_initial_prompt(input).await?
            } else {
                self.generate_continuation_prompt(input, &execution_context).await?
            };

            // Get response from Claude
            if let Some(client) = &self.claude_client {
                let assistant_response = client.chat(&prompt).await?;

                // Add the assistant's reasoning to the conversation
                if !full_conversation.is_empty() {
                    full_conversation.push_str("\n\n---\n\n");
                }
                full_conversation.push_str(&assistant_response);

                // Process any tool calls in the response
                let tool_results = self.execute_tools_from_response(&assistant_response).await?;

                if !tool_results.is_empty() {
                    // Add tool results to both the full conversation and execution context
                    full_conversation.push_str("\n\n**Tool Results:**\n");
                    full_conversation.push_str(&tool_results);

                    // Update execution context for next iteration
                    execution_context.push_str(&format!("\nPrevious iteration:\nAssistant thought: {}\nTool results: {}\n",
                        assistant_response, tool_results));

                    // Check if the assistant indicated they want to continue
                    continue_loop = self.should_continue_chain(&assistant_response);
                } else {
                    // No tools used, conversation is likely complete
                    continue_loop = false;
                }

                // Store complete response in memory (will be updated in final iteration)
                self.memory.add_assistant_message(assistant_response, None);
            } else {
                return Err(crate::error::AppError::Storage("Claude client not configured".to_string()));
            }
        }

        // Save memory to disk after complete execution
        self.save_memory()?;

        Ok(full_conversation)
    }

    async fn generate_initial_prompt(&self, user_input: &str) -> AppResult<String> {
        let tools_description = self.registry.generate_enhanced_tools_description();
        let conversation_context = self.memory.get_context_for_prompt(true);
        let recent_tools = self.memory.get_recent_tool_usage();

        let recent_tools_hint = if !recent_tools.is_empty() {
            format!("Recently used tools: {}\n", recent_tools.join(", "))
        } else {
            String::new()
        };

        let prompt = format!(
            r#"You are a helpful AI assistant with access to various tools for managing notes and tasks. Your personality should be conversational, helpful, and similar to Claude Code's style.

{}Available Tools:
{}

{}Current User Request: {}

## Chain of Thought Instructions:

You will work through this request step by step, potentially using multiple tools to complete the task. Think through your approach and execute tools as needed.

**Execution Pattern:**
1. **Analyze the request** and explain your understanding
2. **Plan your approach** - what steps will you take?
3. **Execute tools** as needed with JSON format
4. **If more actions are needed**, indicate this clearly in your response
5. **Continue until the task is complete**

**Tool Usage Format:**
```json
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        "param1": "value1"
    }}
}}
```

**Continuation Signals:**
If you need to continue with more actions after seeing tool results, include phrases like:
- "Let me also..."
- "I need to..."
- "Next, I'll..."
- "Additionally..."

**Conversational Style:**
- Be natural and explain your thinking process
- Show your reasoning for each step
- Explain what you're doing and why
- Continue until the user's request is fully satisfied

Start by analyzing the request and explaining your approach.
"#,
            conversation_context, tools_description, recent_tools_hint, user_input
        );

        Ok(prompt)
    }

    async fn generate_continuation_prompt(&self, original_request: &str, execution_context: &str) -> AppResult<String> {
        let tools_description = self.registry.generate_enhanced_tools_description();
        let conversation_context = self.memory.get_context_for_prompt(true);

        let prompt = format!(
            r#"{}

Available Tools:
{}

## Continuation Context:

Original User Request: {}

Execution History:
{}

## Continuation Instructions:

You are continuing to work on the user's original request. Based on what you've already done:

1. **Review** what has been accomplished so far
2. **Determine** if the original request is fully satisfied
3. **If more actions are needed**:
   - Explain what you need to do next
   - Execute the appropriate tools with JSON format
   - Use continuation signals like "Let me also...", "Next, I'll...", etc.
4. **If the task is complete**:
   - Provide a natural conclusion
   - Summarize what was accomplished
   - Don't include any tool JSON

**Tool Usage Format (if needed):**
```json
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        "param1": "value1"
    }}
}}
```

**Your Goal:** Complete the user's original request fully. Continue if more actions would be helpful.
"#,
            conversation_context, tools_description, original_request, execution_context
        );

        Ok(prompt)
    }

    async fn execute_tools_from_response(&mut self, response: &str) -> AppResult<String> {
        let mut all_results = Vec::new();

        // Extract ALL JSON objects from the response
        let json_objects = self.extract_all_json_from_response(response);

        for json_str in json_objects {
            // Try parsing as single tool call
            if let Ok(tool_call) = serde_json::from_str::<Value>(&json_str) {
                // Check if it has the tool structure
                if tool_call.get("tool").is_some() && tool_call.get("action").is_some() {
                    let result = self.execute_tool_call(&tool_call).await?;
                    all_results.push(result);
                }
            }
            // Try parsing as array of tool calls
            else if let Ok(tool_calls) = serde_json::from_str::<Vec<Value>>(&json_str) {
                for call in tool_calls {
                    if call.get("tool").is_some() && call.get("action").is_some() {
                        let result = self.execute_tool_call(&call).await?;
                        all_results.push(result);
                    }
                }
            }
        }

        Ok(all_results.join("\n"))
    }

    fn should_continue_chain(&self, response: &str) -> bool {
        let response_lower = response.to_lowercase();

        // Check for explicit continuation signals
        response_lower.contains("let me also") ||
        response_lower.contains("i'll also") ||
        response_lower.contains("next, i'll") ||
        response_lower.contains("additionally") ||
        response_lower.contains("i need to") ||
        response_lower.contains("i should also") ||
        response_lower.contains("now i'll") ||
        response_lower.contains("then i'll")
    }

    async fn process_response(&mut self, response: &str) -> AppResult<String> {
        let (result, _) = self.process_response_with_continuation(response).await?;
        Ok(result)
    }

    async fn process_response_with_continuation(&mut self, response: &str) -> AppResult<(String, bool)> {
        let mut full_response = response.to_string();
        let mut has_tool_results = false;

        // Try to extract JSON from Claude's response
        if let Some(json_str) = self.extract_json_from_response(response) {
            // Try parsing as single tool call
            if let Ok(tool_call) = serde_json::from_str::<Value>(&json_str) {
                let tool_result = self.execute_tool_call(&tool_call).await?;
                full_response = format!("{}\n\n**Tool Result:** {}", response, tool_result);
                has_tool_results = true;
            }
            // Try parsing as array of tool calls
            else if let Ok(tool_calls) = serde_json::from_str::<Vec<Value>>(&json_str) {
                let mut tool_results = Vec::new();
                for call in tool_calls {
                    let result = self.execute_tool_call(&call).await?;
                    tool_results.push(result);
                }
                let combined_results = tool_results.join("\n");
                full_response = format!("{}\n\n**Tool Results:** {}", response, combined_results);
                has_tool_results = true;
            }
        }

        // Determine if we should continue the loop based on keywords in the response
        let should_continue = has_tool_results &&
            (response.to_lowercase().contains("let me also") ||
             response.to_lowercase().contains("i'll also") ||
             response.to_lowercase().contains("next") ||
             response.to_lowercase().contains("additionally"));

        Ok((full_response, should_continue))
    }

    fn extract_all_json_from_response(&self, response: &str) -> Vec<String> {
        let mut json_objects = Vec::new();
        let mut chars = response.char_indices().peekable();
        let mut start_idx = None;
        let mut brace_count = 0;
        let mut in_string = false;
        let mut escape_next = false;

        while let Some((i, ch)) = chars.next() {
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
                        json_objects.push(json_str.to_string());
                        start_idx = None;
                    }
                }
                _ => {}
            }
        }

        json_objects
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
            // Show confirmation prompt
            if !self.confirm_tool_execution(tool_name, action, parameters)? {
                return Ok("Tool execution cancelled by user.".to_string());
            }

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

    // Memory persistence
    fn save_memory(&self) -> AppResult<()> {
        let memory_path = ConversationMemory::get_memory_file_path();
        self.memory.save_to_file(&memory_path)
    }

    // User confirmation for tool execution
    fn confirm_tool_execution(&self, tool_name: &str, action: &str, parameters: &Value) -> AppResult<bool> {
        // Format parameters in a readable way
        let params_formatted = if parameters.is_null() {
            "none".to_string()
        } else {
            serde_json::to_string_pretty(parameters)
                .unwrap_or_else(|_| parameters.to_string())
        };

        println!("\n🤖 AI Agent wants to execute a tool:");
        println!("   Tool: {}", tool_name);
        println!("   Action: {}", action);
        println!("   Parameters: {}", params_formatted);
        print!("\nDo you want to proceed? [y/N]: ");

        io::stdout().flush().map_err(|e|
            crate::error::AppError::Storage(format!("Failed to flush stdout: {}", e)))?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e|
            crate::error::AppError::Storage(format!("Failed to read user input: {}", e)))?;

        let answer = input.trim().to_lowercase();
        Ok(answer == "y" || answer == "yes")
    }
}