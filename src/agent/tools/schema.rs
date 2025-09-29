use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub actions: Vec<ActionSchema>,
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Internal,  // CRUD operations on local data
    External,  // API calls to external services
    System,    // System-level operations
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSchema {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    pub returns: ReturnSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterSchema {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<Value>,
    pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String { max_length: Option<usize> },
    Number { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i64>, max: Option<i64> },
    Boolean,
    Array { item_type: Box<ParameterType> },
    Object { properties: Vec<ParameterSchema> },
    Date,
    DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub pattern: Option<String>,       // Regex pattern
    pub enum_values: Option<Vec<Value>>, // Allowed values
    pub custom: Option<String>,        // Custom validation description
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnSchema {
    pub description: String,
    pub return_type: ParameterType,
    pub possible_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub description: String,
    pub user_request: String,
    pub tool_call: Value,
    pub expected_result: String,
}

impl ToolSchema {
    pub fn to_prompt_format(&self) -> String {
        let mut prompt = String::new();

        prompt.push_str(&format!("### {} Tool ({})\n", self.name,
            match self.category {
                ToolCategory::Internal => "Internal CRUD",
                ToolCategory::External => "External API",
                ToolCategory::System => "System",
            }
        ));

        prompt.push_str(&format!("**Description**: {}\n\n", self.description));

        prompt.push_str("**Available Actions**:\n");
        for action in &self.actions {
            prompt.push_str(&format!("- `{}`: {}\n", action.name, action.description));

            if !action.parameters.is_empty() {
                prompt.push_str("  Parameters:\n");
                for param in &action.parameters {
                    let required = if param.required { " **(required)**" } else { " (optional)" };
                    let type_desc = self.format_parameter_type(&param.param_type);
                    prompt.push_str(&format!("  - `{}` ({}): {}{}\n",
                        param.name, type_desc, param.description, required));

                    if let Some(default) = &param.default_value {
                        prompt.push_str(&format!("    Default: `{}`\n", default));
                    }

                    if let Some(validation) = &param.validation {
                        if let Some(pattern) = &validation.pattern {
                            prompt.push_str(&format!("    Pattern: `{}`\n", pattern));
                        }
                        if let Some(enum_vals) = &validation.enum_values {
                            prompt.push_str(&format!("    Allowed values: {:?}\n", enum_vals));
                        }
                    }
                }
            }

            prompt.push_str(&format!("  Returns: {} - {}\n",
                self.format_parameter_type(&action.returns.return_type),
                action.returns.description));

            if !action.returns.possible_errors.is_empty() {
                prompt.push_str(&format!("  Possible errors: {}\n",
                    action.returns.possible_errors.join(", ")));
            }

            prompt.push('\n');
        }

        if !self.examples.is_empty() {
            prompt.push_str("**Examples**:\n");
            for example in &self.examples {
                prompt.push_str(&format!("- {}\n", example.description));
                prompt.push_str(&format!("  User: \"{}\"\n", example.user_request));
                prompt.push_str(&format!("  Tool call: `{}`\n", example.tool_call));
                prompt.push_str(&format!("  Result: \"{}\"\n\n", example.expected_result));
            }
        }

        prompt
    }

    fn format_parameter_type(&self, param_type: &ParameterType) -> String {
        match param_type {
            ParameterType::String { max_length } => {
                if let Some(max) = max_length {
                    format!("string(max: {})", max)
                } else {
                    "string".to_string()
                }
            }
            ParameterType::Number { min, max } => {
                match (min, max) {
                    (Some(min_val), Some(max_val)) => format!("number({}-{})", min_val, max_val),
                    (Some(min_val), None) => format!("number(min: {})", min_val),
                    (None, Some(max_val)) => format!("number(max: {})", max_val),
                    (None, None) => "number".to_string(),
                }
            }
            ParameterType::Integer { min, max } => {
                match (min, max) {
                    (Some(min_val), Some(max_val)) => format!("integer({}-{})", min_val, max_val),
                    (Some(min_val), None) => format!("integer(min: {})", min_val),
                    (None, Some(max_val)) => format!("integer(max: {})", max_val),
                    (None, None) => "integer".to_string(),
                }
            }
            ParameterType::Boolean => "boolean".to_string(),
            ParameterType::Array { item_type } => {
                format!("array<{}>", self.format_parameter_type(item_type))
            }
            ParameterType::Object { properties: _ } => "object".to_string(),
            ParameterType::Date => "date (YYYY-MM-DD)".to_string(),
            ParameterType::DateTime => "datetime (ISO 8601)".to_string(),
        }
    }
}