use crate::Lens;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Migration {
    pub schema: String,
    pub version: usize,
    pub ops: Vec<Lens>,
}
