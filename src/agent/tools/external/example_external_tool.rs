// Example external tool - demonstrates the pattern for adding new tools
// To add a new tool:
// 1. Create a new .rs file in this directory
// 2. Implement the Tool trait
// 3. The tool will be automatically discovered and made available to the agent

use async_trait::async_trait;
use serde_json::Value;
use crate::agent::{Tool, ToolAction};
use crate::error::AppResult;

pub struct ExampleTool;

impl ExampleTool {
    pub fn new() -> AppResult<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Tool for ExampleTool {
    fn name(&self) -> &str {
        "example"
    }

    fn description(&self) -> &str {
        "An example tool demonstrating the extensible architecture"
    }

    fn actions(&self) -> Vec<ToolAction> {
        vec![
            ToolAction::new("hello", "Say hello with a custom message")
                .with_parameter("name", "Name to greet", false, "string"),

            ToolAction::new("calculate", "Perform a simple calculation")
                .with_parameter("operation", "Operation: add, subtract, multiply, divide", true, "string")
                .with_parameter("a", "First number", true, "number")
                .with_parameter("b", "Second number", true, "number"),
        ]
    }

    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String> {
        match action {
            "hello" => {
                let name = parameters["name"].as_str().unwrap_or("World");
                Ok(format!("Hello, {}! This is from the example tool.", name))
            }
            "calculate" => {
                let operation = parameters["operation"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing operation".to_string()))?;
                let a = parameters["a"].as_f64()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing or invalid number 'a'".to_string()))?;
                let b = parameters["b"].as_f64()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing or invalid number 'b'".to_string()))?;

                let result = match operation {
                    "add" => a + b,
                    "subtract" => a - b,
                    "multiply" => a * b,
                    "divide" => {
                        if b == 0.0 {
                            return Err(crate::error::AppError::Storage("Division by zero".to_string()));
                        }
                        a / b
                    }
                    _ => return Err(crate::error::AppError::Storage(format!("Unknown operation: {}", operation))),
                };

                Ok(format!("{} {} {} = {}", a, operation, b, result))
            }
            _ => Err(crate::error::AppError::Storage(format!("Unknown action: {}", action)))
        }
    }
}