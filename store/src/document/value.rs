use std::collections::BTreeMap;

use serde_json::json;

pub const NULL: Value = Value::Null;

#[derive(Debug, PartialOrd, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Number(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Number(v.into())
    }
}

impl From<u32> for Value {
    fn from(v: u32) -> Self {
        Self::Number(v.into())
    }
}

impl From<&Value> for serde_json::Value {
    fn from(value: &Value) -> Self {
        match value {
            Value::Number(num) => json!(num),
            Value::String(string) => json!(string),
            Value::Bool(bool) => json!(bool),
            Value::Null => json!(null),
        }
    }
}

impl TryFrom<wasm_bindgen::JsValue> for Value {
    type Error = ValueError;

    fn try_from(value: wasm_bindgen::JsValue) -> Result<Self, Self::Error> {
        if let Some(num) = value.as_f64() {
            Ok(Self::Number(num))
        } else if let Some(bool) = value.as_bool() {
            Ok(Self::Bool(bool))
        } else if let Some(string) = value.as_string() {
            Ok(Self::String(string))
        } else {
            Err(ValueError::Unsupported)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("Unsupported JS type")]
    Unsupported,
}
