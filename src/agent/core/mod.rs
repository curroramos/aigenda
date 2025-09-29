use crate::agent::ToolRegistry;
use crate::error::AppResult;
use serde_json::Value;

pub struct Agent {
    registry: ToolRegistry,
    claude_client: Option<crate::ai::claude::ClaudeClient>,
}

impl Agent {
    pub fn new() -> AppResult<Self> {
        let mut registry = ToolRegistry::new();

        // Auto-discover and register all tools
        registry.auto_discover_tools()?;

        Ok(Self {
            registry,
            claude_client: None,
        })
    }

    pub fn with_claude_client(mut self, client: crate::ai::claude::ClaudeClient) -> Self {
        self.claude_client = Some(client);
        self
    }

    pub async fn execute_command(&self, input: &str) -> AppResult<String> {
        // Generate dynamic prompt with available tools
        let prompt = self.generate_prompt(input).await?;

        // Send to Claude API
        if let Some(client) = &self.claude_client {
            let response = client.chat(&prompt).await?;
            self.process_response(&response).await
        } else {
            Err(crate::error::AppError::Storage("Claude client not configured".to_string()))
        }
    }

    async fn generate_prompt(&self, user_input: &str) -> AppResult<String> {
        let tools_description = self.registry.generate_tools_description();

        let prompt = format!(
            r#"You are an AI assistant with access to various tools for managing notes and other tasks.

Available Tools:
{}

User Request: {}

Please analyze the user's request and determine which tools to use. For each tool call, respond with JSON in this format:
{{
    "tool": "tool_name",
    "action": "action_name",
    "parameters": {{
        // tool-specific parameters
    }}
}}

If multiple tool calls are needed, provide them as a JSON array.
"#,
            tools_description, user_input
        );

        Ok(prompt)
    }

    async fn process_response(&self, response: &str) -> AppResult<String> {
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

    async fn execute_tool_call(&self, call: &Value) -> AppResult<String> {
        let tool_name = call["tool"].as_str().ok_or_else(||
            crate::error::AppError::Storage("Missing tool name".to_string()))?;

        let action = call["action"].as_str().ok_or_else(||
            crate::error::AppError::Storage("Missing action".to_string()))?;

        let parameters = &call["parameters"];

        if let Some(tool) = self.registry.get_tool(tool_name) {
            tool.execute(action, parameters).await
        } else {
            Err(crate::error::AppError::Storage(format!("Unknown tool: {}", tool_name)))
        }
    }

    pub fn list_available_tools(&self) -> Vec<String> {
        self.registry.list_tools()
    }
}