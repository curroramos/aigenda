use crate::error::AppResult;

/// Trait for handling streaming responses during agent execution
pub trait StreamingHandler: Send + Sync {
    /// Called when the agent receives a response from the LLM
    fn on_llm_response(&mut self, response: &str) -> AppResult<()>;

    /// Called before a tool is about to be executed
    fn on_tool_about_to_execute(&mut self, tool_name: &str, action: &str, parameters: &serde_json::Value) -> AppResult<()>;

    /// Called after a tool has been executed
    fn on_tool_executed(&mut self, tool_name: &str, action: &str, result: &str, success: bool) -> AppResult<()>;

    /// Called when requesting permission for tool execution
    fn request_tool_permission(&mut self, tool_name: &str, action: &str, parameters: &serde_json::Value) -> AppResult<bool>;

    /// Called at the start of a new iteration in the chain
    fn on_iteration_start(&mut self, iteration: usize) -> AppResult<()>;

    /// Called at the end of an iteration
    fn on_iteration_end(&mut self, iteration: usize, result: &str) -> AppResult<()>;
}

/// Default console streaming handler that outputs to stdout
pub struct ConsoleStreamingHandler;

impl ConsoleStreamingHandler {
    pub fn new() -> Self {
        Self
    }
}

impl StreamingHandler for ConsoleStreamingHandler {
    fn on_llm_response(&mut self, response: &str) -> AppResult<()> {
        println!("\n🤖 AI Response:\n{}\n", response);
        Ok(())
    }

    fn on_tool_about_to_execute(&mut self, tool_name: &str, action: &str, _parameters: &serde_json::Value) -> AppResult<()> {
        println!("⚡ Executing tool: {} -> {}", tool_name, action);
        Ok(())
    }

    fn on_tool_executed(&mut self, tool_name: &str, action: &str, result: &str, success: bool) -> AppResult<()> {
        let status = if success { "✅" } else { "❌" };
        println!("{} Tool result for {} -> {}:", status, tool_name, action);
        if !result.is_empty() {
            println!("{}\n", result);
        }
        Ok(())
    }

    fn request_tool_permission(&mut self, tool_name: &str, action: &str, parameters: &serde_json::Value) -> AppResult<bool> {
        use std::io::{self, Write};

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

    fn on_iteration_start(&mut self, iteration: usize) -> AppResult<()> {
        if iteration > 1 {
            println!("\n🔄 Starting iteration {} of the chain...\n", iteration);
        }
        Ok(())
    }

    fn on_iteration_end(&mut self, iteration: usize, _result: &str) -> AppResult<()> {
        println!("✨ Iteration {} completed.\n", iteration);
        Ok(())
    }
}