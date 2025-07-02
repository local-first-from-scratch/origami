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
    pub async fn init() -> Result<Self, idb::Error> {
        let database = DatabaseBuilder::new("ops")
            .add_object_store(
                ObjectStoreBuilder::new("row")
                    .auto_increment(false)
                    .key_path(Some(KeyPath::new_array(["table", "id"])))
                    .add_index(
                        IndexBuilder::new("by_table".to_string(), KeyPath::new_single("table"))
                            .unique(false)
                            .multi_entry(false),
                    ),
            )
            .add_object_store(
                ObjectStoreBuilder::new("field")
                    .auto_increment(false)
                    .key_path(Some(KeyPath::new_array(["table", "row_id", "field_name"])))
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

    pub async fn new_row(&self, row: Row) -> Result<(), IDBError> {
        let tx = self
            .database
            .transaction(&["row"], TransactionMode::ReadWrite)?;

        let row_store = tx.object_store("row")?;
        row_store.add(&serde_wasm_bindgen::to_value(&row)?, None)?;

        tx.await?;

        Ok(())
    }

    pub async fn get_rows(&self, table: &str) -> Result<Vec<Row>, IDBError> {
        let tx = self
            .database
            .transaction(&["row"], TransactionMode::ReadOnly)?;

        let row_store = tx.object_store("row")?;
        let by_table = row_store.index("by_table")?;

        let cursor = by_table
            .open_cursor(Some(Query::Key(table.into())), Some(CursorDirection::Next))?
            .await?
            .ok_or(IDBError::MissingCursor)?;

        let mut out = Vec::new();
        while let Some(raw_row) = cursor.next(None)?.await? {
            out.push(serde_wasm_bindgen::from_value(raw_row.value()?)?);
        }

        Ok(out)
    }

    // pub async fn get_fields(&self, rows: Vec<uuid::Uuid>) -> Result<Vec<Field>, IDBError> {
    //     todo!()
    // }
}

#[derive(Debug, thiserror::Error)]
pub enum IDBError {
    #[error("IndexedDB error: {0}")]
    Idb(#[from] idb::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_wasm_bindgen::Error),

    #[error("Missing cursor while reading")]
    MissingCursor,
}

impl From<IDBError> for JsValue {
    fn from(val: IDBError) -> Self {
        JsValue::from_str(&val.to_string())
    }
}
