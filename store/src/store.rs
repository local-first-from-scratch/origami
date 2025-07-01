use crate::idb::{IDBError, IDBStorage};
use wasm_bindgen::prelude::*;

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
impl Store {}
