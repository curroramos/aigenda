use crate::agent::memory::{ToolCall, ToolResult};
use crate::agent::confirmation::ConfirmationHandler;
use crate::agent::json_parser::JsonParser;
use crate::agent::streaming::StreamingHandler;
use crate::agent::ToolRegistry;
use crate::error::AppResult;
use chrono::Utc;
use serde_json::Value;
use std::time::Instant;

#[cfg(feature = "ai")]
use uuid::Uuid;

/// Handles tool execution with confirmation and tracking
pub struct ToolExecutor {
    confirmation_handler: ConfirmationHandler,
    json_parser: JsonParser,
}

impl ToolExecutor {
    pub fn new() -> Self {
        Self {
            confirmation_handler: ConfirmationHandler::new(),
            json_parser: JsonParser::new(),
        }
    }

    /// Executes all tool calls found in a response with streaming support
    pub async fn execute_tools_from_response_streaming<H>(
        &mut self,
        response: &str,
        registry: &ToolRegistry,
        streaming_handler: &mut H,
    ) -> AppResult<(Vec<ToolCall>, Vec<ToolResult>, String)>
    where
        H: StreamingHandler,
    {
        let tool_calls = self.json_parser.parse_tool_calls(response);

        if tool_calls.is_empty() {
            return Ok((Vec::new(), Vec::new(), String::new()));
        }

        let mut executed_calls = Vec::new();
        let mut tool_results = Vec::new();
        let mut result_strings = Vec::new();

        for call in tool_calls.iter() {
            if let (Some(tool_name), Some(action)) = (
                call.get("tool").and_then(|t| t.as_str()),
                call.get("action").and_then(|a| a.as_str()),
            ) {
                let parameters = call.get("parameters").unwrap_or(&Value::Null);

                // Request permission dynamically for each tool
                let confirmed = streaming_handler.request_tool_permission(tool_name, action, parameters)?;

                if confirmed {
                    streaming_handler.on_tool_about_to_execute(tool_name, action, parameters)?;

                    let (tool_call, tool_result, result_str) =
                        self.execute_single_tool(call, registry).await?;

                    streaming_handler.on_tool_executed(tool_name, action, &result_str, tool_result.success)?;

                    executed_calls.push(tool_call);
                    tool_results.push(tool_result);
                    result_strings.push(result_str);
                } else {
                    let cancelled_msg = format!("Tool execution cancelled by user: {} -> {}", tool_name, action);
                    result_strings.push(cancelled_msg.clone());
                    streaming_handler.on_tool_executed(tool_name, action, &cancelled_msg, false)?;
                }
            }
        }

        Ok((executed_calls, tool_results, result_strings.join("\n")))
    }

    /// Legacy method for backward compatibility
    pub async fn execute_tools_from_response(
        &mut self,
        response: &str,
        registry: &ToolRegistry,
    ) -> AppResult<(Vec<ToolCall>, Vec<ToolResult>, String)> {
        let tool_calls = self.json_parser.parse_tool_calls(response);

        if tool_calls.is_empty() {
            return Ok((Vec::new(), Vec::new(), String::new()));
        }

        let confirmations = self.confirmation_handler.confirm_multiple_tools(&tool_calls)?;

        let mut executed_calls = Vec::new();
        let mut tool_results = Vec::new();
        let mut result_strings = Vec::new();

        for (call, confirmed) in tool_calls.iter().zip(confirmations.iter()) {
            if *confirmed {
                let (tool_call, tool_result, result_str) =
                    self.execute_single_tool(call, registry).await?;

                executed_calls.push(tool_call);
                tool_results.push(tool_result);
                result_strings.push(result_str);
            } else {
                result_strings.push("Tool execution cancelled by user.".to_string());
            }
        }

        Ok((executed_calls, tool_results, result_strings.join("\n")))
    }

    /// Executes a single tool call
    async fn execute_single_tool(
        &self,
        call: &Value,
        registry: &ToolRegistry,
    ) -> AppResult<(ToolCall, ToolResult, String)> {
        let tool_name = call["tool"].as_str()
            .ok_or_else(|| crate::error::AppError::Storage("Missing tool name".to_string()))?;

        let action = call["action"].as_str()
            .ok_or_else(|| crate::error::AppError::Storage("Missing action".to_string()))?;

        let parameters = &call["parameters"];

        if let Some(tool) = registry.get_tool(tool_name) {
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

            let result_string = result?;
            Ok((tool_call, tool_result, result_string))
        } else {
            Err(crate::error::AppError::Storage(format!("Unknown tool: {}", tool_name)))
        }
    }
}