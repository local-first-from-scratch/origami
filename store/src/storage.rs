use crate::op::{Field, Row};
use uuid::Uuid;

pub mod idb;
pub use idb::{IDBError, IDBStorage};

pub trait Storage {
    type Error;

    /// Get the row IDs for all tables in the database.
    async fn get_rows(&self, table: &str) -> Result<Vec<Row>, Self::Error>;

    /// Get the fields for all specified rows in the database.
    async fn get_fields(&self, rows: Vec<Uuid>) -> Result<Vec<Field>, Self::Error>;
}
