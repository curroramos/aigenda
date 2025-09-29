use clap::Parser;
use aigenda::app::App;
use aigenda::cli::Cli;
use aigenda::error::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = App::new()?;
    app.run(cli)
}
