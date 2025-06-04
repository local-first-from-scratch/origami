use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Lens {
    Add(AddRemove),
    Remove(AddRemove),
    Rename(Rename),

    // Objects
    Extract(ExtractEmbed),
    Embed(ExtractEmbed),

    // Lists
    Head(WrapHead),
    Wrap(WrapHead),

    // Navigation
    In(In),
    Map(Map),

    // Conversion
    Convert(Convert),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AddRemove {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: jtd::SerdeSchema,
    #[serde(default = "null")]
    pub default: Value,
}

fn null() -> Value {
    Value::Null
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Rename {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExtractEmbed {
    pub host: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct WrapHead {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct In {
    pub name: String,
    pub ops: Vec<Lens>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Map {
    pub ops: Vec<Lens>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Convert {
    pub name: String,
    pub from_type: jtd::SerdeSchema,
    pub to_type: jtd::SerdeSchema,
    pub forward: HashMap<Value, Value>,
    pub reverse: HashMap<Value, Value>,
}

impl Lens {
    pub fn reversed(&self) -> Self {
        match self {
            Lens::Add(add_remove) => Lens::Remove(add_remove.clone()),
            Lens::Remove(add_remove) => Lens::Add(add_remove.clone()),
            Lens::Rename(Rename { from, to }) => Lens::Rename(Rename {
                from: to.clone(),
                to: from.clone(),
            }),
            Lens::Extract(extract_embed) => Lens::Embed(extract_embed.clone()),
            Lens::Embed(extract_embed) => Lens::Embed(extract_embed.clone()),
            Lens::Head(wrap_head) => Lens::Wrap(wrap_head.clone()),
            Lens::Wrap(wrap_head) => Lens::Head(wrap_head.clone()),
            Lens::In(..) => self.clone(),
            Lens::Map(..) => self.clone(),
            Lens::Convert(Convert {
                name,
                from_type,
                to_type,
                forward,
                reverse,
            }) => Lens::Convert(Convert {
                name: name.clone(),
                from_type: to_type.clone(),
                to_type: from_type.clone(),
                forward: reverse.clone(),
                reverse: forward.clone(),
            }),
        }
    }
}
