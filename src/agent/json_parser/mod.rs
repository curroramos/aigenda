use serde_json::Value;

/// Handles JSON extraction from agent responses
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }

    /// Extracts all JSON objects from a text response
    pub fn extract_all_json(&self, response: &str) -> Vec<String> {
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

    /// Extracts first JSON object from response (legacy method)
    pub fn extract_first_json(&self, response: &str) -> Option<String> {
        self.extract_all_json(response).into_iter().next()
    }

    /// Validates if a JSON object is a valid tool call
    pub fn is_valid_tool_call(&self, json_obj: &Value) -> bool {
        json_obj.get("tool").and_then(|t| t.as_str()).is_some() &&
        json_obj.get("action").and_then(|a| a.as_str()).is_some()
    }

    /// Parses response into valid tool calls
    pub fn parse_tool_calls(&self, response: &str) -> Vec<Value> {
        let mut valid_calls = Vec::new();

        for json_str in self.extract_all_json(response) {
            // Try parsing as single tool call
            if let Ok(tool_call) = serde_json::from_str::<Value>(&json_str) {
                if self.is_valid_tool_call(&tool_call) {
                    valid_calls.push(tool_call);
                }
            }
            // Try parsing as array of tool calls
            else if let Ok(tool_calls) = serde_json::from_str::<Vec<Value>>(&json_str) {
                for call in tool_calls {
                    if self.is_valid_tool_call(&call) {
                        valid_calls.push(call);
                    }
                }
            }
        }

        valid_calls
    }
}