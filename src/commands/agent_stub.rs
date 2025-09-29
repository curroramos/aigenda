// Stub for AI agent when AI features are disabled
use crate::error::AppResult;

pub async fn handle_agent_command(_prompt: Vec<String>) -> AppResult<()> {
    eprintln!("❌ AI command requires AI features. Rebuild with: cargo build --features ai");
    std::process::exit(1);
}