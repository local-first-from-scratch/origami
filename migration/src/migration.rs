use crate::lens::Lens;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Migration {
    base: Option<String>,
    ops: Vec<Lens>,
}
