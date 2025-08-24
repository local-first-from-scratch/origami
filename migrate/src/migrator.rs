use crate::{Lens, Migration, Schema, lens};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct Migrator {
    paths: BTreeMap<String, BTreeMap<(usize, usize), Vec<Lens>>>,
}

impl Migrator {
    pub fn add_migration(&mut self, migration: Migration) {
        let Migration {
            schema,
            version,
            ops,
        } = migration;

        let schema_entry = self.paths.entry(schema).or_default();

        if version == 0 {
            todo!("Raise an error if version is 0; that's reserved for the blank schema");
        }

        schema_entry.insert(
            (version, version - 1),
            ops.iter().rev().map(|op| op.reversed()).collect(),
        );
        schema_entry.insert((version - 1, version), ops);
    }

    pub fn migration_path(&self, schema: &str, from: usize, to: usize) -> Option<Vec<&Lens>> {
        let mut out = Vec::new();

        if from == to {
            return None;
        }

        let direction = if from < to {
            Direction::Up
        } else {
            Direction::Down
        };

        let mut current = from;

        if let Some(paths) = self.paths.get(schema) {
            while current != to {
                match paths.get(&(current, direction.tick(current))) {
                    Some(path) => {
                        out.extend(path);
                        current = direction.tick(current);
                    }
                    // TODO: say why. This is an error case.
                    None => return None,
                }
            }
        } else {
            // TODO: say why. This is an error case.
            return None;
        };

        Some(out)
    }

    pub fn schema(&self, schema: &str, version: usize) -> Result<Schema, Error> {
        let mut out = Schema::default();

        for lens in self
            .migration_path(schema, 0, version)
            .ok_or_else(|| Error::MigrationPathNotFound(schema.to_string(), version))?
        {
            lens.transform_schema(&mut out)
                .map_err(Error::CouldNotApply)?;
        }

        Ok(out)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("could not find path to migration ({0}.{1})")]
    MigrationPathNotFound(String, usize),
    #[error("could not apply operation: {0}")]
    CouldNotApply(lens::Error),
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
}

impl Direction {
    fn tick(&self, n: usize) -> usize {
        match self {
            Direction::Up => n.saturating_add(1),
            Direction::Down => n.saturating_sub(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Field, Type, Value};

    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! lens {
        ($patch:tt) => {
            serde_json::from_value::<Lens>(serde_json::json!($patch)).unwrap()
        };
    }

    #[test]
    fn migration_path() {
        let mut migrator = Migrator::default();

        let lens_a = lens!({"add": {
            "name": "a",
            "type": "string",
            "nullable": true,
        }});
        let migration_a = Migration {
            schema: "test".into(),
            version: 1,
            ops: vec![lens_a.clone()],
        };
        migrator.add_migration(migration_a.clone());

        let lens_b = lens!({"rename": {
            "from": "a",
            "to": "b"
        }});
        let migration_b = Migration {
            schema: "test".into(),
            version: 2,
            ops: vec![lens_b.clone()],
        };
        migrator.add_migration(migration_b.clone());

        let lens_c = lens!({"rename": {
            "from": "b",
            "to": "c"
        }});
        let migration_c = Migration {
            schema: "test".into(),
            version: 3,
            ops: vec![lens_c.clone()],
        };
        migrator.add_migration(migration_c.clone());

        assert_eq!(
            Some(vec![&lens_a, &lens_b, &lens_c]),
            migrator.migration_path("test", 0, 3)
        );

        println!("========================");

        assert_eq!(
            Some(vec![&lens_c.reversed(), &lens_b.reversed()]),
            migrator.migration_path("test", 3, 1)
        );
    }

    #[test]
    fn schema_missing() {
        let migrator = Migrator::default();

        assert_eq!(
            migrator.schema("nope", 1),
            Err(Error::MigrationPathNotFound("nope".into(), 1))
        )
    }

    #[test]
    fn schema_conflict() {
        let mut migrator = Migrator::default();

        let same_lens = lens!({"add": {
            "name": "a",
            "type": "string",
            "nullable": true,
        }});

        migrator.add_migration(Migration {
            schema: "test".into(),
            version: 1,
            ops: vec![same_lens.clone()],
        });
        migrator.add_migration(Migration {
            schema: "test".into(),
            version: 2,
            ops: vec![same_lens],
        });

        let err = migrator.schema("test", 2).unwrap_err();

        assert!(
            matches!(err, Error::CouldNotApply(..)),
            "Expected CouldNotApply, got {err:?}"
        );
    }

    #[test]
    fn schema_success() {
        let mut migrator = Migrator::default();
        migrator.add_migration(Migration {
            schema: "test".into(),
            version: 1,
            ops: vec![lens!({"add": {
                "name": "a",
                "type": "string",
                "nullable": true,
            }})],
        });

        assert_eq!(
            migrator.schema("test", 1),
            Ok(Schema::from([(
                "a",
                Field {
                    type_: Type::Nullable(Box::new(Type::String)),
                    default: Value::Null,
                }
            )]))
        )
    }
}
