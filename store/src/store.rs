use crate::idb::{IDBError, IDBStorage};
use crate::op::Row;
use crate::timestamp::Timestamp;
use migrate::Value;
use std::collections::BTreeMap;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::JsString;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"

export type TypeMap = Record<string, any>;

export function store<T extends TypeMap>(schemas: Record<keyof T, string>): Promise<Store<T>>;

export class Store<T extends TypeMap> {
  insert<K extends keyof T>(table: K, id: string, data: T[K]): Promise<void>;
  list<K extends keyof T>(table: K): Promise<T[K][]>;
  // get<K extends keyof T>(table: K, id: string): Promise<T[K]>;
  // update<K extends keyof T>(table: K, id: String, updater: (current: T[K]) => void): void;
}
"#;

#[wasm_bindgen(skip_typescript)]
pub struct Store {
    schemas: BTreeMap<String, String>,
    storage: IDBStorage,
}

/// Technical reason this is separate: store initialization needs to be done asynchronously
/// because it involves interacting with the browser's IndexedDB API. Putting
/// that in the object's constructor would make things look really weird on the
/// JavaScript side.
#[wasm_bindgen(skip_typescript)]
pub async fn store(schemas: JsValue) -> Result<Store, StoreError> {
    Ok(Store::new(
        serde_wasm_bindgen::from_value(schemas)?,
        IDBStorage::init().await?,
    ))
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("Invalid schema. Schemas must be an object with string keys and values.")]
    Schema(#[from] serde_wasm_bindgen::Error),
    #[error("IDB error: {0}")]
    Idb(#[from] IDBError),
}

impl From<StoreError> for JsValue {
    fn from(val: StoreError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}

impl Store {
    fn new(schemas: BTreeMap<String, String>, storage: IDBStorage) -> Self {
        Self { schemas, storage }
    }

    pub async fn insert(
        &self,
        table: String,
        data: BTreeMap<String, Value>,
    ) -> Result<Uuid, StoreError> {
        let id = Uuid::now_v7();

        self.storage
            .new_row(Row {
                table,
                id,
                added: Timestamp::new(0, Uuid::nil()),
                removed: None,
            })
            .await?;

        Ok(id)
    }
}

#[wasm_bindgen]
impl Store {
    #[wasm_bindgen(js_name = "insert")]
    pub async fn js_insert(
        &self,
        table_js: JsString,
        data: JsValue,
    ) -> Result<JsString, StoreError> {
        Ok(self
            .insert(table_js.into(), serde_wasm_bindgen::from_value(data)?)
            .await?
            .to_string()
            .into())
    }

    pub async fn list(&self, table_js: JsString) -> Result<JsValue, StoreError> {
        let table: String = table_js.into();
        let rows = self.storage.get_rows(&table).await?;

        Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
    }
}
