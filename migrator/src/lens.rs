use crate::value;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Lens {
    Add(AddRemoveField),
    Remove(AddRemoveField),
    Rename { from: String, to: String },
    // TODO: value conversion
}

impl Lens {
    pub fn reversed(&self) -> Self {
        match self {
            Lens::Add(lens) => Lens::Remove(lens.clone()),
            Lens::Remove(lens) => Lens::Add(lens.clone()),
            Lens::Rename { from, to } => Lens::Rename {
                from: to.clone(),
                to: from.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct AddRemoveField {
    name: String,
    #[serde(rename = "type")]
    _type: value::Type,
    default: Option<value::Value>,
    #[serde(default)]
    nullable: bool,
}
