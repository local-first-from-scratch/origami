use crate::op::Row;
use crate::storage::Storage;
use crate::timestamp::Timestamp;
use migrate::{Migrator, Value};
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
        data: BTreeMap<String, Value>,
    ) -> Result<Uuid, Error<S::Error>> {
        let schema = self
            .table_to_schema
            .get(&table)
            .ok_or_else(|| Error::TableNotFound(table.clone()))?;

        let id = Uuid::now_v7();

        self.storage
            .store_row(Row {
                table,
                id,
                added: Timestamp::new(0, Uuid::nil()),
                removed: None,
            })
            .await?;

        Ok(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error<E: std::error::Error> {
    #[error("Storage error: {0}")]
    Storage(#[from] E),

    #[error("Schema not found for table {0}")]
    TableNotFound(String),
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

    fn init() -> Store<MemoryStorage> {
        let mut migrator = Migrator::new();
        migrator.add_migration(migrate::Migration {
            id: "test.v1".to_string(),
            base: None,
            ops: vec![],
        });

        Store::new(
            migrator,
            BTreeMap::from([("test".into(), "test.v1".into())]),
            MemoryStorage::default(),
        )
    }

    #[tokio::test]
    async fn test_insert_success() {
        let mut store = init();

        store
            .insert("test".to_string(), BTreeMap::new())
            .await
            .unwrap();
    }
}
