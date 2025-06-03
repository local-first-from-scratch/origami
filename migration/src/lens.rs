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
    // pub type: type
    pub default: Value,
    pub nullable: bool,
    // pub items: type
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
    // pub from_type: type,
    // pub to_type: type,
    pub forward: HashMap<Value, Value>,
    pub reverse: HashMap<Value, Value>,
}
