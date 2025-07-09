use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat, Error};
use migrate::{Migration, Migrator};
use std::collections::BTreeMap;
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
        /// The name of the eventual schema at this migration (e.g. `users.20250625`)
        id: String,
        /// The predecessor of this migration (e.g. `users.20250401`)
        #[clap(long, short)]
        base: Option<String>,
    },

    /// Show the schema at a given ID
    Schema { id: String },
}

impl App {
    fn run(&self) -> Result<(), Error> {
        match &self.command {
            Command::New { id, base } => {
                if !self.dir.exists() {
                    std::fs::create_dir_all(&self.dir).wrap_err_with(|| {
                        format!(
                            "Could not create migrations directory at {}",
                            self.dir.display()
                        )
                    })?;
                }

                let blank = Migration {
                    id: id.clone(),
                    base: base.clone(),
                    ops: Vec::new(),
                };

                let file_path = self.dir.join(format!("{}.json", id.replace("/", "_")));
                let file = std::fs::File::create(&file_path).wrap_err_with(|| {
                    format!("Could not create migration file at {}", file_path.display())
                })?;
                serde_json::to_writer_pretty(file, &blank).wrap_err_with(|| {
                    format!("Could not write migration to {}", file_path.display())
                })?;

                println!("Migration created at {}", file_path.display());

                Ok(())
            }
            Command::Schema { id } => {
                let mut migrator = Migrator::new();
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

                let mut schema = jtd::Schema::Properties {
                    definitions: BTreeMap::new(),
                    metadata: BTreeMap::new(),
                    nullable: false,
                    properties: BTreeMap::new(),
                    optional_properties: BTreeMap::new(),
                    properties_is_present: true,
                    additional_properties: false,
                };

                for lens in migrator
                    .migration_path(None, id)
                    .wrap_err("could not find path to migration")?
                {
                    lens.transform_jtd(&mut schema)
                        .wrap_err("could not apply operation")?;
                }

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
