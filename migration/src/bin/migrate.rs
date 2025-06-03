use clap::{Parser, Subcommand};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use migration::{migration::Migration, migrator::Migrator};
use std::fs::{create_dir_all, read_dir};
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, global = true, default_value = "migrations")]
    migrations_dir: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    /// Initialize the migration directory
    Init,

    /// Get a JSON schema at a given migration
    Schema {
        /// The name of the schema to export
        name: String,
    },
}

impl Cli {
    fn get_migrator(&self) -> Result<Migrator> {
        let mut migrator = Migrator::new();

        let dir = read_dir(&self.migrations_dir).wrap_err("could not read migrations directory")?;
        for entry_res in dir {
            let entry = entry_res?;

            if !entry.file_type()?.is_file() {
                continue;
            }

            let file = std::fs::File::open(&entry.path())
                .wrap_err_with(|| format!("could not open file {}", entry.path().display()))?;
            let migration: Migration = serde_json::from_reader(file).wrap_err_with(|| {
                format!("could not parse migration {}", entry.path().display())
            })?;

            let name = entry
                .path()
                .file_stem()
                .wrap_err("could not convert file name to migration name")?
                .to_string_lossy()
                .to_string();

            migrator.add_migration(name, migration);
        }

        Ok(migrator)
    }

    fn run(&self) -> Result<()> {
        match &self.command {
            Command::Init => {
                println!("Initializing migration directory");
                create_dir_all(&self.migrations_dir)
                    .wrap_err("could not create migrations directory")?;
            }

            Command::Schema { name } => {
                let migrator = self.get_migrator()?;

                let path = migrator
                    .migration_path(None, name)
                    .wrap_err_with(|| format!("could not find migration path to {}", name))?;

                println!(
                    "{}",
                    path.iter()
                        .map(|m| m.name.clone())
                        .collect::<Vec<String>>()
                        .join(" -> ")
                )
            }
        }

        Ok(())
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = cli.run() {
        eprintln!("{:?}", err);
        std::process::exit(1);
    };
}
