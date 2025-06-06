use clap::{Parser, Subcommand};
use color_eyre::{
    Result,
    eyre::{Context, ContextCompat},
};
use jtd::Schema;
use migration::{migration::Migration, migrator::Migrator};
use std::path::PathBuf;
use std::{
    collections::BTreeMap,
    fs::{create_dir_all, read_dir},
};

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

            let file = std::fs::File::open(entry.path())
                .wrap_err_with(|| format!("could not open file {}", entry.path().display()))?;
            let migration: Migration = serde_json::from_reader(file).wrap_err_with(|| {
                format!("could not parse migration {}", entry.path().display())
            })?;

            migrator.add_migration(migration);
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

                let mut dest = Schema::Empty {
                    definitions: BTreeMap::new(),
                    metadata: BTreeMap::new(),
                };

                for lens in path {
                    lens.transform_jtd(&mut dest)
                        .wrap_err("could not transform schema")?;
                }

                let json = serde_json::to_string_pretty(&dest.into_serde_schema())?;
                println!("{}", json);
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
