use crate::lens::Lens;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Migration {
    pub base: Option<String>,
    pub ops: Vec<Lens>,
}
