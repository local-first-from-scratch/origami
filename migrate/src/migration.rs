use crate::Lens;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Migration {
    pub id: String,
    pub base: Option<String>,
    pub ops: Vec<Lens>,
}
