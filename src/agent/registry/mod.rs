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

    pub fn generate_enhanced_tools_description(&self) -> String {
        let mut description = String::new();

        // Group tools by category
        let mut internal_tools = Vec::new();
        let mut external_tools = Vec::new();
        let mut system_tools = Vec::new();

        for tool in self.tools.values() {
            match tool.category() {
                crate::agent::ToolCategory::Internal => internal_tools.push(tool),
                crate::agent::ToolCategory::External => external_tools.push(tool),
                crate::agent::ToolCategory::System => system_tools.push(tool),
            }
        }

        // Add internal tools (CRUD)
        if !internal_tools.is_empty() {
            description.push_str("## Internal Tools (Local CRUD Operations)\n");
            for tool in internal_tools {
                description.push_str(&tool.get_schema().to_prompt_format());
                description.push_str("\n");
            }
        }

        // Add external tools (APIs)
        if !external_tools.is_empty() {
            description.push_str("## External Tools (API Integrations)\n");
            for tool in external_tools {
                description.push_str(&tool.get_schema().to_prompt_format());
                description.push_str("\n");
            }
        }

        // Add system tools
        if !system_tools.is_empty() {
            description.push_str("## System Tools\n");
            for tool in system_tools {
                description.push_str(&tool.get_schema().to_prompt_format());
                description.push_str("\n");
            }
        }

        description
    }

    pub fn get_enhanced_schemas(&self) -> Vec<String> {
        self.tools.values()
            .map(|tool| tool.get_schema().to_prompt_format())
            .collect()
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