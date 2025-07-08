use crate::op::Row;

use super::Storage;

pub struct MemoryStorage {
    rows: Vec<Row>,
}

impl Storage for MemoryStorage {
    type Error = Error;

    async fn store_row(&mut self, row: Row) -> Result<(), Self::Error> {
        self.rows.push(row);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
