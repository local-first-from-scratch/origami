use crate::lens::Lens;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Migration {
    pub name: String,
    pub base: Option<String>,
    pub ops: Vec<Lens>,
}
