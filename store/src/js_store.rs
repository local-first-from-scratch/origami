use crate::storage::idb::{self, IDBStorage};
use crate::store::{self, Store};
use std::collections::BTreeMap;
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

#[wasm_bindgen(skip_typescript, js_name = "Store")]
pub struct JsStore {
    store: Store<IDBStorage>,
}

/// Technical reason this is separate: store initialization needs to be done asynchronously
/// because it involves interacting with the browser's IndexedDB API. Putting
/// that in the object's constructor would make things look really weird on the
/// JavaScript side.
#[wasm_bindgen(skip_typescript)]
pub async fn store(schemas: JsValue) -> Result<JsStore, Error> {
    Ok(JsStore::new(
        serde_wasm_bindgen::from_value(schemas)?,
        IDBStorage::init().await?,
    ))
}

impl JsStore {
    pub fn new(schemas: BTreeMap<String, String>, storage: IDBStorage) -> Self {
        JsStore {
            store: Store::new(schemas, storage),
        }
    }
}

#[wasm_bindgen]
impl JsStore {
    #[wasm_bindgen(js_name = "insert")]
    pub async fn js_insert(
        &mut self,
        table_js: JsString,
        data: JsValue,
    ) -> Result<JsString, Error> {
        Ok(self
            .store
            .insert(table_js.into(), serde_wasm_bindgen::from_value(data)?)
            .await?
            .to_string()
            .into())
    }

    pub async fn list(&self, _table_js: JsString) -> Result<JsValue, Error> {
        // let table: String = table_js.into();
        // let rows = self.storage.get_rows(&table).await?;

        // Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid schema. Schemas must be an object with string keys and values.")]
    Serde(#[from] serde_wasm_bindgen::Error),
    #[error("IndexedDB error: {0}")]
    Idb(#[from] idb::Error),
    #[error("Store error: {0}")]
    Store(#[from] store::Error<idb::Error>),
}

impl From<Error> for JsValue {
    fn from(val: Error) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
