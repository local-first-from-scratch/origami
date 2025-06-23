#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
