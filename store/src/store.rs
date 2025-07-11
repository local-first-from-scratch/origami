use crate::op::{Field, Row};
use crate::storage::{RWTransaction, Storage};
use crate::timestamp::Timestamp;
use migrate::{Migrator, Value, migrator, type_};
use std::collections::BTreeMap;
use std::fmt::Display;
use uuid::Uuid;
use wasm_bindgen::JsValue;

pub struct Store<S: Storage> {
    migrator: Migrator,
    table_to_schema: BTreeMap<String, String>,
    storage: S,
}

impl<S: Storage> Store<S> {
    pub fn new(migrator: Migrator, table_to_schema: BTreeMap<String, String>, storage: S) -> Self {
        Self {
            migrator,
            table_to_schema,
            storage,
        }
    }

    pub async fn insert(
        &mut self,
        table: String,
        mut data: BTreeMap<String, Value>,
    ) -> Result<Uuid, Error<S::Error>> {
        let schema_name = self
            .table_to_schema
            .get(&table)
            .ok_or_else(|| Error::TableNotFound(table.clone()))?;

        let schema = self.migrator.schema(schema_name).map_err(Error::Schema)?;

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
                    table: table.clone(),
                    row_id: id,
                    field_name: name,
                    timestamp: Timestamp::new(0, Uuid::nil()), // TODO: Implement timestamp generation
                    schema: schema_name.clone(),
                    value,
                })
                .await
                .map_err(Error::Storage)?;
            }
        }

        tx.store_row(Row {
            table,
            id,
            added: Timestamp::new(0, Uuid::nil()), // TODO: Implement timestamp generation
            removed: None,
        })
        .await
        .map_err(Error::Storage)?;

        tx.commit().await.map_err(Error::Storage)?;

        Ok(id)
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error<E: std::error::Error> {
    #[error("Storage error: {0}")]
    Storage(#[from] E),

    #[error("Schema not found for table {0}")]
    TableNotFound(String),

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
    use crate::storage::memory::MemoryStorage;
    use migrate::{AddRemoveField, Lens, Type};

    fn init() -> Store<MemoryStorage> {
        let mut migrator = Migrator::new();
        migrator.add_migration(migrate::Migration {
            id: "test.v1".to_string(),
            base: None,
            ops: vec![Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "default".into(),
            })],
        });

        Store::new(
            migrator,
            BTreeMap::from([("test".into(), "test.v1".into())]),
            MemoryStorage::default(),
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

        assert_eq!(row.table, "test");
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

        assert_eq!(field.table, "test");
        assert_eq!(field.row_id, row.id);
        assert_eq!(field.field_name, "test");
        assert_eq!(field.schema, "test.v1");
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
