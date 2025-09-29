#[cfg(feature = "ai")]
use crate::agent::{Agent, ConsoleStreamingHandler};
#[cfg(feature = "ai")]
use crate::ai::claude::ClaudeClient;
use crate::error::AppResult;

#[cfg(feature = "ai")]
pub async fn handle_agent_command(prompt: Vec<String>) -> AppResult<()> {
    let input = prompt.join(" ");

    if input.trim().is_empty() {
        println!("Usage: aigenda ai <your natural language command>");
        println!("Example: aigenda ai \"add a note about today's meeting\"");
        println!("Example: aigenda ai \"show me my notes from yesterday\"");
        return Ok(());
    }

    // Initialize the agent
    let mut agent = Agent::new()?;

    // Try to initialize Claude client if API key is available
    if let Ok(claude_client) = ClaudeClient::new() {
        agent = agent.with_claude_client(claude_client);

        println!("🤖 Processing your request...");

        let mut streaming_handler = ConsoleStreamingHandler::new();
        match agent.execute_command_streaming(&input, &mut streaming_handler).await {
            Ok(_response) => {
                println!("\n✅ Command completed successfully!");
            }
            Err(e) => {
                eprintln!("❌ Error executing command: {}", e);

                // Fallback: show available tools
                println!("\n📋 Available tools:");
                for tool in agent.list_available_tools() {
                    println!("  • {}", tool);
                }
            }
        }
    } else {
        println!("⚠️  Claude API key not found. Set ANTHROPIC_API_KEY environment variable.");
        println!("   For now, showing available tools:\n");

        println!("📋 Available tools:");
        for tool in agent.list_available_tools() {
            println!("  • {}", tool);
        }

        println!("\n💡 Once you set your API key, you can use natural language commands like:");
        println!("   aigenda ai \"add a note about today's meeting\"");
        println!("   aigenda ai \"show me my notes from yesterday\"");
        println!("   aigenda ai \"update my note from today\"");
    }

    Ok(())
}

