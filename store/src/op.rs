use crate::timestamp::Timestamp;
use migrate::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Row {
    pub table: String,
    pub id: Uuid,
    pub added: Timestamp,
    pub removed: Option<Timestamp>,
}

pub struct Field {
    pub table: String,
    pub row_id: Uuid,
    pub field_name: String,
    pub timestamp: Timestamp,
    pub schema: String,
    pub value: Value,
}
