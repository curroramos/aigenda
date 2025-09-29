use async_trait::async_trait;
use serde_json::Value;
use crate::error::AppResult;

pub mod notes;
pub mod external;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn actions(&self) -> Vec<ToolAction>;
    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String>;
}

#[derive(Clone, Debug)]
pub struct ToolAction {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
}

#[derive(Clone, Debug)]
pub struct ToolParameter {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub param_type: String, // "string", "number", "boolean", "array", "object"
}

impl ToolAction {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters: Vec::new(),
        }
    }

    pub fn with_parameter(mut self, name: &str, description: &str, required: bool, param_type: &str) -> Self {
        self.parameters.push(ToolParameter {
            name: name.to_string(),
            description: description.to_string(),
            required,
            param_type: param_type.to_string(),
        });
        self
    }
}