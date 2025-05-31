use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRemove {
    name: String,
    // type: type
    default: Value,
    required: bool,
    // items: type
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rename {
    from: String,
    to: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractEmbed {
    host: String,
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WrapHead {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct In {
    name: String,
    lens: Vec<Lens>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Map {
    lens: Vec<Lens>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Convert {
    name: String,
    mapping: Vec<(Value, Value)>,
}
