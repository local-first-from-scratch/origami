use crate::hlc::Hlc;
use migrate::Value;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Row {
    pub schema: String,
    pub id: Uuid,
    pub added: Hlc,
    pub removed: Option<Hlc>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Field {
    pub schema: String,
    pub row_id: Uuid,
    pub field_name: String,
    pub timestamp: Hlc,
    pub schema_version: usize,
    pub value: Value,
}
