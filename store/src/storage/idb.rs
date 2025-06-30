use super::Storage;
use idb::{
    Database, KeyPath,
    builder::{DatabaseBuilder, IndexBuilder, ObjectStoreBuilder},
};

struct IDBStorage {
    database: Database,
}

#[derive(Debug, thiserror::Error)]
enum InitError {
    #[error("IDB error: {0}")]
    IDB(#[from] idb::Error),
}

impl IDBStorage {
    async fn init() -> Result<Self, InitError> {
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
                ObjectStoreBuilder::new("fieldSet")
                    .auto_increment(false)
                    .key_path(Some(KeyPath::new_array(["table", "row_id", "field_name"])))
                    .add_index(
                        IndexBuilder::new("by_table".to_string(), KeyPath::new_single("table"))
                            .unique(false)
                            .multi_entry(false),
                    )
                    .add_index(
                        IndexBuilder::new(
                            "by_row".to_string(),
                            KeyPath::new_array(["table", "row_id"]),
                        )
                        .unique(false)
                        .multi_entry(false),
                    ),
            )
            .build()
            .await?;

        Ok(Self { database })
    }
}

impl Storage for IDBStorage {}
