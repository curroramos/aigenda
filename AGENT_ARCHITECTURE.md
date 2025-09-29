# AI Agent Architecture

This document describes the scalable AI agent architecture implemented in aigenda.

## Overview

The AI agent system is built with a dynamic tool discovery architecture that allows new tools to be added simply by creating new files. The agent automatically discovers and registers tools, making them available for natural language commands.

## Architecture

```
src/agent/
├── core/           # Core agent logic and Claude API integration
├── tools/          # Tool definitions and implementations
│   ├── notes/      # Built-in notes CRUD tool
│   └── external/   # Directory for external/custom tools
├── registry/       # Tool registration and discovery system
└── prompt/         # Dynamic prompt generation (future)
```

## Components

### 1. Agent Core (`src/agent/core/mod.rs`)

The main `Agent` struct that:
- Manages tool registry
- Handles Claude API communication
- Processes natural language commands
- Executes tool calls based on Claude's response

### 2. Tool Registry (`src/agent/registry/mod.rs`)

Automatically discovers and registers tools:
- Scans for tool implementations
- Maintains tool catalog
- Generates dynamic tool descriptions for Claude

### 3. Tool Trait (`src/agent/tools/mod.rs`)

The `Tool` trait that all tools must implement:

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn actions(&self) -> Vec<ToolAction>;
    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String>;
}
```

### 4. Notes Tool (`src/agent/tools/notes/mod.rs`)

Built-in CRUD operations for notes:
- `create`: Add new notes
- `read`: Retrieve notes by date or recent
- `update`: Modify existing notes
- `delete`: Remove notes

## Usage

### Basic Commands

```bash
# Build with AI features
cargo build --features ai

# Set your Claude API key
export ANTHROPIC_API_KEY="your-key-here"

# Use the agent
cargo run --features ai -- ai "add a note about today's meeting"
cargo run --features ai -- ai "show me my notes from yesterday"
cargo run --features ai -- ai "update my first note from today"
```

### Example Interactions

```bash
$ cargo run --features ai -- ai "add a note saying I completed the AI agent implementation"
🤖 Processing your request...
✅ Note added successfully for 2025-09-28

$ cargo run --features ai -- ai "show me today's notes"
🤖 Processing your request...
✅ Notes for 2025-09-28:
1. [20:45] I completed the AI agent implementation

$ cargo run --features ai -- ai "calculate 25 + 17"
🤖 Processing your request...
✅ 25 add 17 = 42
```

## Adding New Tools

To add a new tool, simply create a new file in `src/agent/tools/external/` and implement the `Tool` trait:

### Example: Weather Tool

Create `src/agent/tools/external/weather_tool.rs`:

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
        "Get current weather information"
    }

    fn actions(&self) -> Vec<ToolAction> {
        vec![
            ToolAction::new("current", "Get current weather for a location")
                .with_parameter("location", "City name", true, "string"),

            ToolAction::new("forecast", "Get weather forecast")
                .with_parameter("location", "City name", true, "string")
                .with_parameter("days", "Number of days (1-7)", false, "number"),
        ]
    }

    async fn execute(&self, action: &str, parameters: &Value) -> AppResult<String> {
        match action {
            "current" => {
                let location = parameters["location"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing location".to_string()))?;
                // Implement weather API call here
                Ok(format!("Current weather in {}: Sunny, 22°C", location))
            }
            "forecast" => {
                let location = parameters["location"].as_str()
                    .ok_or_else(|| crate::error::AppError::Storage("Missing location".to_string()))?;
                let days = parameters["days"].as_u64().unwrap_or(3);
                // Implement forecast API call here
                Ok(format!("{}-day forecast for {}: Mostly sunny", days, location))
            }
            _ => Err(crate::error::AppError::Storage(format!("Unknown action: {}", action)))
        }
    }
}
```

### Register the Tool

Update `src/agent/tools/external/mod.rs`:

```rust
use crate::agent::ToolRegistry;
use crate::error::AppResult;
use std::sync::Arc;

mod weather_tool;

pub fn register_all_external_tools(registry: &mut ToolRegistry) -> AppResult<()> {
    // Register weather tool
    let weather_tool = Arc::new(weather_tool::WeatherTool::new()?);
    registry.register_tool(weather_tool);

    Ok(())
}
```

### Update Tool Registration

In `src/agent/registry/mod.rs`, update the auto-discovery to call the registration function:

```rust
pub fn auto_discover_tools(&mut self) -> AppResult<()> {
    // Register built-in notes tool
    let notes_tool = Arc::new(crate::agent::tools::notes::NotesTool::new()?);
    self.register_tool(notes_tool);

    // Register external tools
    crate::agent::tools::external::register_all_external_tools(self)?;

    Ok(())
}
```

Now your weather tool will be automatically available:

```bash
cargo run --features ai -- ai "what's the weather in San Francisco?"
```

## Key Features

### 1. Dynamic Tool Discovery
- Tools are automatically registered when the agent starts
- No need to manually update central configuration
- Just implement the `Tool` trait and the agent finds it

### 2. Natural Language Interface
- Claude interprets user intent and maps to tool actions
- Automatic parameter extraction and validation
- Flexible command structure

### 3. Extensible Architecture
- Add new tools without modifying core code
- Tools can be simple functions or complex integrations
- Consistent interface for all tools

### 4. Type Safety
- Full Rust type safety throughout
- Async/await support for I/O operations
- Proper error handling and propagation

## Future Enhancements

1. **Plugin System**: Dynamic loading of tools from separate crates
2. **Tool Chaining**: Allow tools to call other tools
3. **Persistent Context**: Remember conversation history
4. **Custom Prompts**: Per-tool prompt customization
5. **Tool Categories**: Group tools by functionality

## Configuration

The agent respects the following environment variables:

- `ANTHROPIC_API_KEY`: Your Claude API key (required)
- `AIGENDA_DATA_DIR`: Custom data directory (optional)

## Security Considerations

- API keys are loaded from environment variables only
- All user input is validated before tool execution
- Tools run with the same permissions as the main process
- No arbitrary code execution from user input

This architecture provides a solid foundation for building sophisticated AI-powered CLI tools with extensible functionality.