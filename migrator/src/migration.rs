use crate::lens::Lens;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Migration {
    pub id: String,
    pub collection: String,
    pub operations: Vec<Lens>,
}
