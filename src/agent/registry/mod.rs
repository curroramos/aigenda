use std::collections::HashMap;
use std::sync::Arc;
use crate::agent::Tool;
use crate::error::AppResult;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register_tool(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    pub fn list_tools(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }

    pub fn generate_tools_description(&self) -> String {
        let mut description = String::new();

        for tool in self.tools.values() {
            description.push_str(&format!("\n## {} Tool\n", tool.name()));
            description.push_str(&format!("Description: {}\n", tool.description()));
            description.push_str("Actions:\n");

            for action in tool.actions() {
                description.push_str(&format!("- {}: {}\n", action.name, action.description));

                if !action.parameters.is_empty() {
                    description.push_str("  Parameters:\n");
                    for param in &action.parameters {
                        let required = if param.required { " (required)" } else { " (optional)" };
                        description.push_str(&format!(
                            "    - {} ({}): {}{}\n",
                            param.name, param.param_type, param.description, required
                        ));
                    }
                }
            }
            description.push('\n');
        }

        description
    }

    pub fn auto_discover_tools(&mut self) -> AppResult<()> {
        // Register built-in notes tool
        let notes_tool = Arc::new(crate::agent::tools::notes::NotesTool::new()?);
        self.register_tool(notes_tool);

        // Register external tools
        crate::agent::tools::external::register_all_external_tools(self)?;

        Ok(())
    }

}