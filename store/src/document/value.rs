use std::collections::BTreeMap;

pub const NULL: Value = Value::Null;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    Map(BTreeMap<String, Value>),
    List(Vec<Value>),
    Null,
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Self::List(v)
    }
}

impl<Inner> From<BTreeMap<String, Inner>> for Value
where
    Inner: Into<Value>,
{
    fn from(orig: BTreeMap<String, Inner>) -> Self {
        let mut transformed = BTreeMap::new();
        for (k, v) in orig.into_iter() {
            transformed.insert(k, v.into());
        }

        Self::Map(transformed)
    }
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

#[wasm_bindgen::prelude::wasm_bindgen]
extern "C" {
    type ObjectExt;

    #[wasm_bindgen(method, indexing_setter)]
    fn set(this: &ObjectExt, key: js_sys::JsString, value: wasm_bindgen::JsValue);
}

// Implementation note: keep `use`s and similar from leaking outside this
// implementation so we can put it behind a flag eventually for server
// compilation.
impl From<Value> for wasm_bindgen::JsValue {
    fn from(value: Value) -> Self {
        use js_sys::{Array, Object};
        use wasm_bindgen::{JsCast, JsValue};

        match value {
            Value::Number(i) => JsValue::from_f64(i),
            Value::String(s) => JsValue::from_str(&s),
            Value::Bool(b) => JsValue::from_bool(b),
            Value::Map(map) => {
                let out = Object::new();

                for (k, v) in map {
                    out.unchecked_ref::<ObjectExt>().set(k.into(), v.into())
                }

                out.into()
            }
            Value::List(values) => {
                let out = Array::new();

                for v in values {
                    out.push(&v.into());
                }

                out.into()
            }
            Value::Null => JsValue::null(),
        }
    }
}
