use super::{ROTransaction, RWTransaction, Storage};
use crate::op::{Field, Row};

#[derive(Default)]
pub struct MemoryStorage {
    pub rows: Vec<Row>,
    pub fields: Vec<Field>,
}

impl Storage for MemoryStorage {
    type Error = Error;

    type RWTransaction<'a>
        = RWMemoryTransaction<'a>
    where
        Self: 'a;

    type ROTransaction<'a>
        = ROMemoryTransaction<'a>
    where
        Self: 'a;

    async fn rw_transaction(&mut self) -> Result<Self::RWTransaction<'_>, Self::Error> {
        Ok(RWMemoryTransaction {
            storage: self,
            rows: Vec::new(),
            fields: Vec::new(),
        })
    }

    async fn ro_transaction(&self) -> Result<Self::ROTransaction<'_>, Self::Error> {
        Ok(ROMemoryTransaction { storage: self })
    }
}

pub struct RWMemoryTransaction<'a> {
    storage: &'a mut MemoryStorage,
    rows: Vec<Row>,
    fields: Vec<Field>,
}

impl<'a> RWTransaction for RWMemoryTransaction<'a> {
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
        self.storage.rows.append(&mut self.rows);
        self.storage.fields.append(&mut self.fields);

        Ok(())
    }

    async fn abort(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub struct ROMemoryTransaction<'a> {
    storage: &'a MemoryStorage,
}

impl<'a> ROTransaction for ROMemoryTransaction<'a> {
    type Error = Error;

    async fn list_rows(&self, schema: &str) -> Result<Vec<Row>, Self::Error> {
        Ok(self
            .storage
            .rows
            .iter()
            .filter(|r| r.schema == schema)
            .cloned()
            .collect())
    }

    async fn list_fields(&self, id: uuid::Uuid) -> Result<Vec<Field>, Self::Error> {
        Ok(self
            .storage
            .fields
            .iter()
            .filter(|f| id == f.row_id)
            .cloned()
            .collect())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {}
