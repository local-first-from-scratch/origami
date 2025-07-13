use clap::Parser;
use color_eyre::eyre::{Context, Error};
use migrate::{Migration, Migrator};
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Parser)]
struct App {
    #[arg(long, default_value = "migrations", global = true)]
    dir: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Create a new migration
    New {
        /// The name of the eventual schema at this migration (e.g. `users`)
        schema: String,
        /// The version of this migration (e.g. 3)
        version: usize,
    },

    /// Show the schema at a given ID
    Schema {
        /// The name of the schema you want to retrieve.
        schema: String,
        /// The version of the schema you want to retrieve.
        version: usize,
    },
}

impl App {
    fn run(&self) -> Result<(), Error> {
        match &self.command {
            Command::New { schema, version } => {
                if !self.dir.exists() {
                    std::fs::create_dir_all(&self.dir).wrap_err_with(|| {
                        format!(
                            "Could not create migrations directory at {}",
                            self.dir.display()
                        )
                    })?;
                }

                let blank = Migration {
                    schema: schema.clone(),
                    version: *version,
                    ops: Vec::new(),
                };

                let file_path = self
                    .dir
                    .join(format!("{}.{version}.json", schema.replace("/", "_")));
                let file = std::fs::File::create(&file_path).wrap_err_with(|| {
                    format!("Could not create migration file at {}", file_path.display())
                })?;
                serde_json::to_writer_pretty(file, &blank).wrap_err_with(|| {
                    format!("Could not write migration to {}", file_path.display())
                })?;

                println!("Migration created at {}", file_path.display());

                Ok(())
            }
            Command::Schema { schema, version } => {
                let mut migrator = Migrator::default();
                for entry in self
                    .dir
                    .read_dir()
                    .wrap_err("could not read migrations directory")?
                {
                    let entry = entry.wrap_err("could not read migration directory entry")?;

                    let file =
                        File::open(entry.path()).wrap_err("could not read migration file")?;

                    let migration = serde_json::from_reader(file)
                        .wrap_err("could not deserialize migration")?;

                    migrator.add_migration(migration);
                }

                let schema: jtd::Schema = migrator
                    .schema(schema, *version)
                    .wrap_err("could not get schema")?
                    .into();

                serde_json::to_writer_pretty(std::io::stdout(), &schema.into_serde_schema())
                    .wrap_err("could not serialize schema")?;

                Ok(())
            }
        }
    }
}

fn main() {
    let app = App::parse();

    if let Err(err) = app.run() {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
}
