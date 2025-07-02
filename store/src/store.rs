use crate::idb::{IDBError, IDBStorage};
use crate::op::Row;
use crate::timestamp::Timestamp;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::JsString;

#[wasm_bindgen]
pub struct Store {
    storage: IDBStorage,
}

/// Initialize a store
///
/// Technical reason: this needs to be done asynchronously because it involves
/// interacting with the browser's IndexedDB API. Putting that in the object's
/// constructor would make things look really weird on the JavaScript side.
#[wasm_bindgen]
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
    pub async fn insert_test(&self) -> Result<(), IDBError> {
        self.storage
            .new_row(Row {
                table: "test".into(),
                id: Uuid::now_v7(),
                added: Timestamp::new(0, Uuid::nil()),
                removed: None,
            })
            .await
    }

    pub async fn get_rows(&self, table_js: JsString) -> Result<JsValue, IDBError> {
        let table: String = table_js.into();
        let rows = self.storage.get_rows(&table).await?;

        Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
    }
}
