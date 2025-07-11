use super::{RWTransaction, Storage};
use crate::op::{Field, Row};

#[derive(Default)]
pub struct MemoryStorage {
    pub rows: Vec<Row>,
    pub fields: Vec<Field>,
}

impl Storage for MemoryStorage {
    type Error = Error;
    type RWTransaction<'a>
        = MemoryTransaction<'a>
    where
        Self: 'a;

    async fn rw_transaction(&mut self) -> Result<Self::RWTransaction<'_>, Self::Error> {
        Ok(MemoryTransaction {
            storage: self,
            rows: Vec::new(),
            fields: Vec::new(),
        })
    }
}

pub struct MemoryTransaction<'a> {
    storage: &'a mut MemoryStorage,
    rows: Vec<Row>,
    fields: Vec<Field>,
}

impl<'a> RWTransaction for MemoryTransaction<'a> {
    type Error = Error;

    async fn store_row(&mut self, row: Row) -> Result<(), Error> {
        self.rows.push(row);
        Ok(())
    }

    async fn store_field(&mut self, field: Field) -> Result<(), Self::Error> {
        self.fields.push(field);
        Ok(())
    }

    async fn commit(mut self) -> Result<(), Self::Error> {
        self.storage.rows.extend(self.rows.drain(..));
        self.storage.fields.extend(self.fields.drain(..));

        Ok(())
    }

    async fn abort(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
