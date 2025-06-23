use crate::value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Lens {
    Add(AddRemoveField),
    Remove(AddRemoveField),
    Rename { old_name: String, new_name: String },
    // TODO: value conversion
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AddRemoveField {
    name: String,
    #[serde(rename = "type")]
    _type: value::Type,
    default: Option<value::Value>,
    nullable: bool,
}

impl Lens {
    fn reverse(&self) -> Self {
        match self {
            Lens::Add(lens) => Lens::Remove(lens.clone()),
            Lens::Remove(lens) => Lens::Add(lens.clone()),
            Lens::Rename { old_name, new_name } => Lens::Rename {
                old_name: new_name.clone(),
                new_name: old_name.clone(),
            },
        }
    }
}
