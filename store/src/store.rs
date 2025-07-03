use crate::idb::{IDBError, IDBStorage};
use crate::op::Row;
use crate::timestamp::Timestamp;
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
    storage: IDBStorage,
}

/// Technical reason this is separate: store initialization needs to be done asynchronously
/// because it involves interacting with the browser's IndexedDB API. Putting
/// that in the object's constructor would make things look really weird on the
/// JavaScript side.
#[wasm_bindgen(skip_typescript)]
pub async fn store() -> Result<Store, IDBError> {
    Ok(Store::from_storage(IDBStorage::init().await?))
}

impl Store {
    fn from_storage(storage: IDBStorage) -> Self {
        Self { storage }
    }
}

#[wasm_bindgen]
impl Store {
    pub async fn insert(&self, table_js: JsString, _data: JsValue) -> Result<(), IDBError> {
        self.storage
            .new_row(Row {
                table: table_js.into(),
                id: Uuid::now_v7(),
                added: Timestamp::new(0, Uuid::nil()),
                removed: None,
            })
            .await
    }

    pub async fn list(&self, table_js: JsString) -> Result<JsValue, IDBError> {
        let table: String = table_js.into();
        let rows = self.storage.get_rows(&table).await?;

        Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
    }
}
