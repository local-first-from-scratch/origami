use crate::clock::Clock;
use crate::hlc::Hlc;
use crate::op::{Field, Row};
use crate::storage::{ROTransaction, RWTransaction, Storage};
use migrate::{Migrator, Value, migrator, type_};
use std::collections::BTreeMap;
use std::fmt::Display;
use uuid::Uuid;
use wasm_bindgen::JsValue;

pub struct Store<S: Storage, C: Clock> {
    migrator: Migrator,
    schema_to_version: BTreeMap<String, usize>,
    storage: S,
    clock: C,
}

impl<S: Storage, C: Clock> Store<S, C> {
    pub fn new(
        migrator: Migrator,
        schema_to_version: BTreeMap<String, usize>,
        storage: S,
        clock: C,
    ) -> Self {
        Self {
            migrator,
            schema_to_version,
            storage,
            clock,
        }
    }

    pub async fn insert(
        &mut self,
        schema_name: String,
        mut data: BTreeMap<String, Value>,
    ) -> Result<Uuid, Error<S::Error>> {
        let schema_version = self
            .schema_to_version
            .get(&schema_name)
            .ok_or_else(|| Error::SchemaNotFound(schema_name.clone()))?;

        let schema = self
            .migrator
            .schema(&schema_name, *schema_version)
            .map_err(Error::Schema)?;

        let mut tx = self
            .storage
            .rw_transaction()
            .await
            .map_err(Error::Storage)?;

        let id = Uuid::now_v7();

        for (name, field) in schema {
            if let Some(value) = data.remove(&name) {
                if let Err(err) = field.type_.validate(&value) {
                    tx.abort().await.map_err(Error::Storage)?;
                    return Err(Error::Validation(name.clone(), err));
                }

                tx.store_field(Field {
                    schema: schema_name.clone(),
                    row_id: id,
                    field_name: name,
                    timestamp: Hlc::zero(),
                    schema_version: *schema_version,
                    value,
                })
                .await
                .map_err(Error::Storage)?;
            }
        }

        tx.store_row(Row {
            schema: schema_name,
            id,
            added: Hlc::zero(), // TODO: Implement timestamp generation
            removed: None,
        })
        .await
        .map_err(Error::Storage)?;

        tx.commit().await.map_err(Error::Storage)?;

        Ok(id)
    }

    pub async fn list(
        &self,
        schema: String,
    ) -> Result<Vec<BTreeMap<String, Value>>, Error<S::Error>> {
        let tx = self.storage.ro_transaction().await?;

        let mut out = Vec::new();

        for row in tx.list_rows(&schema).await? {
            if row.removed.is_some_and(|removed| removed > row.added) {
                continue;
            }

            let mut obj = BTreeMap::new();

            for field in tx.list_fields(row.id).await? {
                // TODO: this is really naive. In particular, it doesn't take
                // operation ordering or migrations into consideration. Also, it
                // might be better to write to a cache and then read from it
                // here instead of reading all the values? Writes are currently
                // much cheaper than reads, but reads will be much more often
                // than writes.
                obj.insert(field.field_name, field.value);
            }

            out.push(obj)
        }

        Ok(out)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error<E: std::error::Error> {
    #[error("Storage error: {0}")]
    Storage(#[from] E),

    #[error("Schema `{0}` not found")]
    SchemaNotFound(String),

    #[error("Error retrieving schema")]
    Schema(migrator::Error),

    #[error("Problem validating data for field {0}: {1}")]
    Validation(String, type_::Error),
}

impl<E: std::error::Error + Display> From<Error<E>> for JsValue {
    fn from(val: Error<E>) -> Self {
        JsValue::from_str(&val.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::system_time::SystemTime;
    use crate::storage::memory::MemoryStorage;
    use migrate::{AddRemoveField, Lens, Type};

    fn init() -> Store<MemoryStorage, SystemTime> {
        let mut migrator = Migrator::default();
        migrator.add_migration(migrate::Migration {
            schema: "test".into(),
            version: 1,
            ops: vec![Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "default".into(),
            })],
        });

        Store::new(
            migrator,
            BTreeMap::from([("test".into(), 1)]),
            MemoryStorage::default(),
            SystemTime,
        )
    }

    #[tokio::test]
    async fn insert_stores_row() {
        let mut store = init();

        store
            .insert("test".to_string(), BTreeMap::new())
            .await
            .unwrap();

        let row = &store.storage.rows[0];

        assert_eq!(row.schema, "test");
        assert_eq!(row.removed, None);
    }

    #[tokio::test]
    async fn insert_stores_field() {
        let mut store = init();

        store
            .insert(
                "test".to_string(),
                BTreeMap::from([("test".into(), "hooray!".into())]),
            )
            .await
            .unwrap();

        let row = &store.storage.rows[0];
        let field = &store.storage.fields[0];

        assert_eq!(field.schema, "test");
        assert_eq!(field.row_id, row.id);
        assert_eq!(field.field_name, "test");
        assert_eq!(field.schema_version, 1);
        assert_eq!(field.value, "hooray!".into());
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let mut store = init();

        let result = store
            .insert(
                "test".to_string(),
                BTreeMap::from([("test".into(), 1.into())]),
            )
            .await;

        assert!(
            matches!(result, Err(Error::Validation(ref name, _)) if name == "test"),
            "Expected validation error for \"test\", got {result:?}"
        );

        // we should have aborted the transaction; no changes to rows or fields
        assert_eq!(store.storage.rows.len(), 0);
        assert_eq!(store.storage.fields.len(), 0);
    }
}
