#[cfg(feature = "ai")]
mod agent;
#[cfg(feature = "ai")]
mod ai;
mod app;
mod cli;
mod commands;
mod config;
mod error;
mod models;
mod storage;

use clap::Parser;

#[cfg(feature = "ai")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let app = app::build_default(cli)?;
    app.run().await?;
    Ok(())
}

#[cfg(not(feature = "ai"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let app = app::build_default(cli)?;

    // Run with basic tokio runtime for non-AI builds
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(app.run())?;

    Ok(())
}
