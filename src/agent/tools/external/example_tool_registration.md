# How to Add a New Tool - Step by Step Example

This guide shows how to add a new tool to the AI agent system using the example weather tool.

## Step 1: Create Your Tool File

Create `src/agent/tools/external/weather.rs`:

```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::agent::tools::{Tool, ToolAction};
use crate::error::AppResult;

pub struct WeatherTool;

impl WeatherTool {
    pub fn new() -> AppResult<Self> {
        Ok(Self)
    }
}

#[async_trait]
impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "weather"
    }

    fn description(&self) -> &str {
        "Get weather information for any city"
    }

    fn actions(&self) -> Vec<ToolAction> {
        vec![
            ToolAction::new("current", "Get current weather conditions")
                .with_parameter("city", "Name of the city", true, "string"),
        ]
    }

    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String> {
        match action {
            "current" => {
                let city = parameters["city"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing city parameter".to_string()))?;

                // For demo purposes, return mock data
                // In reality, you'd call a weather API here
                Ok(format!("Weather in {}: Sunny, 22°C, light breeze", city))
            }
            _ => Err(crate::error::AppError::Storage(format!("Unknown action: {}", action)))
        }
    }
}
```

## Step 2: Register the Tool

Update `src/agent/tools/external/mod.rs`:

```rust
use crate::agent::ToolRegistry;
use crate::error::AppResult;
use std::sync::Arc;

// Import your tool modules here
mod weather;
// mod your_other_tool;

pub fn register_all_external_tools(registry: &mut ToolRegistry) -> AppResult<()> {
    // Register weather tool
    let weather_tool = Arc::new(weather::WeatherTool::new()?);
    registry.register_tool(weather_tool);

    // Register other tools here
    // let other_tool = Arc::new(your_other_tool::OtherTool::new()?);
    // registry.register_tool(other_tool);

    Ok(())
}
```

## Step 3: Update the Registry

Update `src/agent/registry/mod.rs` to call your registration function:

In the `auto_discover_tools` method, make sure it calls the external tool registration:

```rust
pub fn auto_discover_tools(&mut self) -> AppResult<()> {
    // Register built-in notes tool
    let notes_tool = Arc::new(crate::agent::tools::notes::NotesTool::new()?);
    self.register_tool(notes_tool);

    // Auto-discover external tools from files
    crate::agent::tools::external::register_all_external_tools(self)?;

    Ok(())
}
```

## Step 4: Test Your Tool

1. Build with AI features:
```bash
cargo build --features ai
```

2. Set your API key:
```bash
export ANTHROPIC_API_KEY="your-key-here"
```

3. Test the tool:
```bash
cargo run --features ai -- ai "what's the weather in Tokyo?"
cargo run --features ai -- ai "check weather for New York"
```

The agent will automatically:
1. Discover your weather tool
2. Include it in the prompt to Claude
3. Execute it when Claude determines it should be used

## Adding More Complex Tools

For more complex tools that need external dependencies, configuration, or state:

```rust
use async_trait::async_trait;
use serde_json::Value;
use crate::agent::tools::{Tool, ToolAction};
use crate::error::AppResult;
use std::collections::HashMap;

pub struct DatabaseTool {
    connection_string: String,
    // Add other state as needed
}

impl DatabaseTool {
    pub fn new() -> AppResult<Self> {
        let connection_string = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./data.db".to_string());

        // Initialize database connection, etc.

        Ok(Self {
            connection_string,
        })
    }

    async fn query_database(&self, query: &str) -> AppResult<String> {
        // Implement database query logic
        Ok(format!("Query result for: {}", query))
    }
}

#[async_trait]
impl Tool for DatabaseTool {
    fn name(&self) -> &str {
        "database"
    }

    fn description(&self) -> &str {
        "Query and manage database records"
    }

    fn actions(&self) -> Vec<ToolAction> {
        vec![
            ToolAction::new("query", "Execute a database query")
                .with_parameter("sql", "SQL query to execute", true, "string"),

            ToolAction::new("tables", "List all tables")
        ]
    }

    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String> {
        match action {
            "query" => {
                let sql = parameters["sql"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing SQL query".to_string()))?;
                self.query_database(sql).await
            }
            "tables" => {
                self.query_database("SELECT name FROM sqlite_master WHERE type='table'").await
            }
            _ => Err(crate::error::AppError::Storage(format!("Unknown action: {}", action)))
        }
    }
}
```

## Key Points

1. **Thread Safety**: Tools must be `Send + Sync` for async execution
2. **Error Handling**: Use `AppResult<T>` for consistent error handling
3. **Parameter Validation**: Always validate parameters from Claude's response
4. **Documentation**: Clear descriptions help Claude choose the right tool and action
5. **Testing**: Test your tools both standalone and through the agent

The agent will automatically make your tools available through natural language commands!