use crate::storage::idb::{IDBError, IDBStorage};
use crate::store::{Store, StoreError};
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
pub async fn store(schemas: JsValue) -> Result<JsStore, StoreError<IDBError>> {
    Ok(JsStore::new(
        serde_wasm_bindgen::from_value(schemas).map_err(StoreError::Schema)?,
        IDBStorage::init().await.map_err(StoreError::Storage)?,
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
        &self,
        table_js: JsString,
        data: JsValue,
    ) -> Result<JsString, StoreError<IDBError>> {
        Ok(self
            .store
            .insert(
                table_js.into(),
                serde_wasm_bindgen::from_value(data).map_err(StoreError::Schema)?,
            )
            .await?
            .to_string()
            .into())
    }

    pub async fn list(&self, _table_js: JsString) -> Result<JsValue, StoreError<IDBError>> {
        // let table: String = table_js.into();
        // let rows = self.storage.get_rows(&table).await?;

        // Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
        todo!()
    }
}
