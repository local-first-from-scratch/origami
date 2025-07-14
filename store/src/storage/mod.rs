pub mod idb;
pub mod memory;

use uuid::Uuid;

use crate::op::{Field, Row};

pub trait Storage {
    type Error: std::error::Error;

    type RWTransaction<'a>: RWTransaction<Error = Self::Error>
    where
        Self: 'a;

    type ROTransaction<'a>: ROTransaction<Error = Self::Error>
    where
        Self: 'a;

    async fn rw_transaction(&mut self) -> Result<Self::RWTransaction<'_>, Self::Error>;
    async fn ro_transaction(&self) -> Result<Self::ROTransaction<'_>, Self::Error>;
}

pub trait RWTransaction {
    type Error: std::error::Error;

    async fn store_row(&mut self, row: Row) -> Result<(), Self::Error>;
    async fn store_field(&mut self, field: Field) -> Result<(), Self::Error>;

    async fn commit(self) -> Result<(), Self::Error>;
    async fn abort(self) -> Result<(), Self::Error>;
}

pub trait ROTransaction {
    type Error: std::error::Error;

    async fn list_rows(&self, schema: &str) -> Result<Vec<Row>, Self::Error>;
    async fn list_fields(&self, id: Uuid) -> Result<Vec<Field>, Self::Error>;
}
