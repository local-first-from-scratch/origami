pub mod idb;

use crate::op::Row;

pub trait Storage {
    type Error: std::error::Error;

    async fn store_row(&self, row: Row) -> Result<(), Self::Error>;
}
