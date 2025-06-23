use crate::lens::Lens;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Migration {
    pub id: String,
    pub base: Option<String>,
    pub ops: Vec<Lens>,
}
