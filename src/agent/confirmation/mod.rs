use crate::error::AppResult;
use serde_json::Value;
use std::io::{self, Write};

/// Handles user confirmation for tool execution
pub struct ConfirmationHandler;

impl ConfirmationHandler {
    pub fn new() -> Self {
        Self
    }

    /// Shows confirmation prompt and gets user input
    pub fn confirm_tool_execution(
        &self,
        tool_name: &str,
        action: &str,
        parameters: &Value,
    ) -> AppResult<bool> {
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

        io::stdout().flush().map_err(|e| {
            crate::error::AppError::Storage(format!("Failed to flush stdout: {}", e))
        })?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| {
            crate::error::AppError::Storage(format!("Failed to read user input: {}", e))
        })?;

        let answer = input.trim().to_lowercase();
        Ok(answer == "y" || answer == "yes")
    }

    /// Shows confirmation for multiple tools
    pub fn confirm_multiple_tools(&self, tool_calls: &[Value]) -> AppResult<Vec<bool>> {
        let mut confirmations = Vec::new();

        for (i, call) in tool_calls.iter().enumerate() {
            if let (Some(tool_name), Some(action)) = (
                call.get("tool").and_then(|t| t.as_str()),
                call.get("action").and_then(|a| a.as_str()),
            ) {
                let parameters = call.get("parameters").unwrap_or(&Value::Null);

                println!("\n--- Tool {} of {} ---", i + 1, tool_calls.len());
                let confirmed = self.confirm_tool_execution(tool_name, action, parameters)?;
                confirmations.push(confirmed);

                if !confirmed {
                    println!("❌ Tool execution cancelled by user.");
                } else {
                    println!("✅ Tool execution approved.");
                }
            } else {
                confirmations.push(false);
            }
        }

        Ok(confirmations)
    }
}