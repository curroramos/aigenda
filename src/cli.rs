use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "aigenda", version, about = "AI-ready daily notes CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a note to today's log
    Add { text: Vec<String> },

    /// List notes (today by default)
    List {
        /// List all days
        #[arg(long)]
        all: bool,
        /// Specific date (YYYY-MM-DD)
        #[arg(long)]
        date: Option<String>,
    },
}
