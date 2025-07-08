use crate::op::Row;
use crate::storage::Storage;
use crate::timestamp::Timestamp;
use migrate::Value;
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::Display;
use uuid::Uuid;
use wasm_bindgen::JsValue;

pub struct Store<S: Storage> {
    table_map: BTreeMap<String, String>,
    storage: S,
}

impl<S: Storage> Store<S> {
    pub fn new(table_map: BTreeMap<String, String>, storage: S) -> Self {
        Self { table_map, storage }
    }

    pub async fn insert(
        &self,
        table: String,
        data: BTreeMap<String, Value>,
    ) -> Result<Uuid, StoreError<S::Error>> {
        let id = Uuid::now_v7();

        self.storage
            .store_row(Row {
                table,
                id,
                added: Timestamp::new(0, Uuid::nil()),
                removed: None,
            })
            .await
            .map_err(StoreError::Storage)?;

        Ok(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError<S: Error> {
    #[error("Invalid schema. Schemas must be an object with string keys and values.")]
    Schema(serde_wasm_bindgen::Error),
    #[error("Storage error: {0}")]
    Storage(S),
}

impl<S: Error + Display> From<StoreError<S>> for JsValue {
    fn from(val: StoreError<S>) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
