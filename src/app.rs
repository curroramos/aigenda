use crate::{
    cli::{Cli, Commands},
    commands::{add, list, agent},
    error::AppResult,
    storage::{fs::FsStorage, Storage},
};

pub struct App<S: Storage> {
    store: S,
    cli: Cli,
}

impl<S: Storage> App<S> {
    pub fn new(store: S, cli: Cli) -> Self {
        Self { store, cli }
    }

    pub async fn run(&self) -> AppResult<()> {
        match &self.cli.command {
            Commands::Add { text } => add::run_add(&self.store, text.clone()),
            Commands::List { all, date } => list::run_list(&self.store, *all, date.clone()),
            Commands::Ai { prompt } => agent::handle_agent_command(prompt.clone()).await,
        }
    }
}

pub fn build_default(cli: Cli) -> AppResult<App<FsStorage>> {
    let store = FsStorage::new()?;
    Ok(App::new(store, cli))
}
