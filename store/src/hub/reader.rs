use migrate::{Lens, Migrator};
use patch::{Path, SetOp};
use serde_json::Value;
use std::collections::{BTreeMap, btree_map::Entry};

pub struct Reader<'m> {
    migrator: &'m Migrator,
    path_cache: BTreeMap<String, Vec<&'m Lens>>,

    target: String,
    values: BTreeMap<Path, Vec<Value>>,
}

impl<'m> Reader<'m> {
    pub fn new(migrations: &'m Migrator, target: String) -> Self {
        Self {
            migrator: migrations,
            path_cache: BTreeMap::new(),

            target,
            values: BTreeMap::new(),
        }
    }

    fn migration_path(&mut self, src: String) -> Result<&Vec<&'m Lens>, Error> {
        match self.path_cache.entry(src.clone()) {
            Entry::Vacant(entry) => {
                let migration_path = self
                    .migrator
                    .migration_path(Some(entry.key()), &self.target)
                    .ok_or_else(|| Error::MigrationFailed(src, self.target.clone()))?;

                Ok(entry.insert(migration_path))
            }
            Entry::Occupied(entry) => {
                // This one's a bit weird. We're returning an immutable borrow
                // from our cache, but using `into_mut` to get it. What's going
                // on here?
                //
                // Well it turns out that on `OccupiedEntry`, `get` returns a
                // borrow that only lives as long as the entry. Other methods
                // are similar—the value doesn't live long enough. `into_mut` is
                // the only method that returns a borrow with the same lifetime
                // as the `BTreeMap` instead of the entry.
                Ok(entry.into_mut())
            }
        }
    }

    pub fn add_patch(&mut self, patch: SetOp) -> Result<(), Error> {
        let SetOp {
            mut path,
            mut value,
            schema,
        } = patch;

        for lens in self.migration_path(schema)? {
            if lens.transform_path(&mut path) {
                return Ok(());
            };

            lens.transform_value(&mut value);
        }

        self.values.entry(path).or_default().push(value);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Cannot find migration path from {0} to {1}")]
    MigrationFailed(String, String),
}
