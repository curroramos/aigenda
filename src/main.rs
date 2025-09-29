mod ai;
mod app;
mod cli;
mod commands;
mod config;
mod error;
mod models;
mod storage;

use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();
    let app = app::build_default(cli)?;
    app.run()?;
    Ok(())
}
