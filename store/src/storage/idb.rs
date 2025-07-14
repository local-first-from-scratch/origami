use super::{RWTransaction, Storage};
use crate::op::Row;
use idb::{
    CursorDirection, Database, KeyPath, Query, TransactionMode,
    builder::{DatabaseBuilder, IndexBuilder, ObjectStoreBuilder},
};
use wasm_bindgen::JsValue;

pub struct IDBStorage {
    database: Database,
}

impl IDBStorage {
    pub async fn init() -> Result<Self, Error> {
        let database = DatabaseBuilder::new("ops")
            .add_object_store(
                ObjectStoreBuilder::new("row")
                    .auto_increment(false)
                    .key_path(Some(KeyPath::new_array(["schema", "id"])))
                    .add_index(
                        IndexBuilder::new("by_schema".to_string(), KeyPath::new_single("schema"))
                            .unique(false)
                            .multi_entry(false),
                    ),
            )
            .add_object_store(
                ObjectStoreBuilder::new("field")
                    .auto_increment(false)
                    .key_path(Some(KeyPath::new_array(["schema", "row_id", "field_name"])))
                    .add_index(
                        IndexBuilder::new("by_row_id".to_string(), KeyPath::new_single("row_id"))
                            .unique(false)
                            .multi_entry(false),
                    ),
            )
            .build()
            .await?;

        Ok(Self { database })
    }

    pub async fn get_rows(&self, schema: &str) -> Result<Vec<Row>, Error> {
        let tx = self
            .database
            .transaction(&["row"], TransactionMode::ReadOnly)?;

        let row_store = tx.object_store("row")?;
        let by_schema = row_store.index("by_schema")?;

        let cursor = by_schema
            .open_cursor(Some(Query::Key(schema.into())), Some(CursorDirection::Next))?
            .await?
            .ok_or(Error::MissingCursor)?;

        let mut out = Vec::new();
        while let Some(raw_row) = cursor.next(None)?.await? {
            out.push(serde_wasm_bindgen::from_value(raw_row.value()?)?);
        }

        Ok(out)
    }
}

impl Storage for IDBStorage {
    type Error = Error;
    type RWTransaction<'a>
        = IDBRWTransaction
    where
        Self: 'a;

    async fn rw_transaction(&mut self) -> Result<Self::RWTransaction<'_>, Self::Error> {
        Ok(IDBRWTransaction(self.database.transaction(
            &["row", "field"],
            TransactionMode::ReadWrite,
        )?))
    }
}

pub struct IDBRWTransaction(idb::Transaction);

impl RWTransaction for IDBRWTransaction {
    type Error = Error;

    async fn store_row(&mut self, row: Row) -> Result<(), Error> {
        let row_store = self.0.object_store("row")?;
        row_store.add(&serde_wasm_bindgen::to_value(&row)?, None)?;

        Ok(())
    }

    async fn store_field(&mut self, field: crate::op::Field) -> Result<(), Self::Error> {
        let field_store = self.0.object_store("field")?;
        field_store.add(&serde_wasm_bindgen::to_value(&field)?, None)?;

        Ok(())
    }

    async fn commit(self) -> Result<(), Self::Error> {
        self.0.await?;

        Ok(())
    }

    async fn abort(self) -> Result<(), Self::Error> {
        self.0.abort()?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IndexedDB error: {0}")]
    Idb(#[from] idb::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),

    #[error("Missing cursor while reading")]
    MissingCursor,
}

impl From<Error> for JsValue {
    fn from(val: Error) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
