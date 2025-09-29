// External tools will be auto-discovered from this directory
// Each tool should implement the Tool trait and register itself

use crate::agent::ToolRegistry;
use crate::error::AppResult;

pub fn register_all_external_tools(_registry: &mut ToolRegistry) -> AppResult<()> {
    // External tools will be registered here automatically
    // This function is called during tool discovery

    // Example: Register the example tool
    // let example_tool = Arc::new(example_external_tool::ExampleTool::new()?);
    // registry.register_tool(example_tool);

    // For now, no external tools are registered by default
    // Add your tools here following the pattern above

    Ok(())
}