use std::collections::BTreeMap;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub enum Value {
    Int(isize),
    Float(f64),
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
        Self::Float(v)
    }
}

impl From<isize> for Value {
    fn from(v: isize) -> Self {
        Self::Int(v)
    }
}
