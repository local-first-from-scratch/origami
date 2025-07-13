use crate::storage::idb::{self, IDBStorage};
use crate::store::{self, Store as GenericStore};
use migrate::{Migration, Migrator};
use std::collections::BTreeMap;
use tokio::sync::RwLock;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::JsString;

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"

export type TypeMap = Record<string, any>;

export function store<T extends TypeMap>(schemas: Record<keyof T, string>, migrations: any[]): Promise<Store<T>>;

export class Store<T extends TypeMap> {
  insert<K extends keyof T>(table: K, data: T[K]): Promise<void>;
  list<K extends keyof T>(table: K): Promise<T[K][]>;
  // get<K extends keyof T>(table: K, id: string): Promise<T[K]>;
  // update<K extends keyof T>(table: K, id: String, updater: (current: T[K]) => void): void;
}
"#;

#[wasm_bindgen(skip_typescript)]
pub struct Store {
    store: RwLock<GenericStore<IDBStorage>>,
}

/// Technical reason this is separate: store initialization needs to be done asynchronously
/// because it involves interacting with the browser's IndexedDB API. Putting
/// that in the object's constructor would make things look really weird on the
/// JavaScript side.
#[wasm_bindgen(skip_typescript)]
pub async fn store(schemas: JsValue, migrations_raw: JsValue) -> Result<Store, Error> {
    console_error_panic_hook::set_once();

    let mut migrator = Migrator::new();
    let migrations: Vec<Migration> =
        serde_wasm_bindgen::from_value(migrations_raw).map_err(Error::Migration)?;
    for migration in migrations {
        migrator.add_migration(migration);
    }

    Ok(Store::new(
        migrator,
        serde_wasm_bindgen::from_value(schemas).map_err(Error::SchemaMapping)?,
        IDBStorage::init().await?,
    ))
}

impl Store {
    pub fn new(migrator: Migrator, schemas: BTreeMap<String, String>, storage: IDBStorage) -> Self {
        Store {
            store: RwLock::new(GenericStore::new(migrator, schemas, storage)),
        }
    }
}

#[wasm_bindgen]
impl Store {
    // Implementation note: all the fields here should borrow self immutably and
    // use the RwLock to read or write as necessary. If we don't do this, we
    // could create both a mutable and immutable borrow in async actions on the
    // JS side and trip the detection against undefined behavior that
    // wasm-bindgen builds into the binary.
    #[wasm_bindgen]
    pub async fn insert(&self, table_js: JsString, data: JsValue) -> Result<JsString, Error> {
        Ok(self
            .store
            .write()
            .await
            .insert(
                table_js.into(),
                serde_wasm_bindgen::from_value(data).map_err(Error::Value)?,
            )
            .await?
            .to_string()
            .into())
    }

    #[wasm_bindgen]
    pub async fn list(&self, _table_js: JsString) -> Result<JsValue, Error> {
        // let table: String = table_js.into();
        // let rows = self.storage.get_rows(&table).await?;

        // Ok(serde_wasm_bindgen::to_value(&rows).unwrap())
        let empty: Vec<()> = Vec::new();
        serde_wasm_bindgen::to_value(&empty).map_err(Error::Value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid migration. Details: {0}")]
    Migration(serde_wasm_bindgen::Error),
    #[error("Invalid schema mapping. Details: {0}")]
    SchemaMapping(serde_wasm_bindgen::Error),
    #[error("Invalid value. Details: {0}")]
    Value(serde_wasm_bindgen::Error),
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
