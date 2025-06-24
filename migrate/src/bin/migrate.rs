use anyhow::{Error, bail};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct App {
    #[arg(long, default_value = "migrations")]
    dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Create a new migration
    New,

    /// Show the schema at a given ID
    Schema { id: String },
}

impl App {
    fn run(&self) -> Result<(), Error> {
        match self.command {
            Command::New => bail!("todo"),
            Command::Schema { .. } => bail!("todo"),
        }
    }
}

fn main() {
    let app = App::parse();
    println!("{app:?}");

    if let Err(err) = app.run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
