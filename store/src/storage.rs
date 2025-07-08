pub mod idb;
pub mod memory;

use crate::op::Row;

pub trait Storage {
    type Error: std::error::Error;

    async fn store_row(&mut self, row: Row) -> Result<(), Self::Error>;
}
