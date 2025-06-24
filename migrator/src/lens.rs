use crate::type_::{SerdeType, Type};
use crate::value;

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
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
pub struct SerdeAddRemoveField {
    name: String,
    #[serde(rename = "type")]
    type_: SerdeType,
    #[serde(default)]
    nullable: bool,
    #[serde(default = "value::Value::null")]
    default: value::Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AddRemoveField {
    name: String,
    type_: Type,
    default: value::Value,
}

impl<'de> serde::Deserialize<'de> for AddRemoveField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let SerdeAddRemoveField {
            name,
            type_,
            nullable,
            default,
        } = SerdeAddRemoveField::deserialize(deserializer)?;

        let final_type = Type::from_serde(type_, nullable);

        final_type
            .validate(&default)
            .map_err(serde::de::Error::custom)?;

        Ok(Self {
            name,
            type_: final_type,
            default,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_::SerdeType;
    use crate::value::Value;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn addremovefield_deserializes_successfully() {
        let deserialized = serde_json::from_value::<AddRemoveField>(json!({
            "name": "test_field",
            "type": "string",
            "nullable": false,
            "default": "default_value"
        }))
        .unwrap();

        assert_eq!(
            AddRemoveField {
                name: "test_field".to_string(),
                type_: Type::from_serde(SerdeType::String, false),
                default: "default_value".into(),
            },
            deserialized
        );
    }

    #[test]
    fn addremovefield_checks_default() {
        let err = serde_json::from_value::<AddRemoveField>(json!({
            "name": "test_field",
            "type": "string",
            "nullable": false,
            "default": null,
        }))
        .unwrap_err();

        assert_eq!("Invalid value for type String: Null", err.to_string(),);
    }

    #[test]
    fn addremovefield_sets_null_as_default() {
        let deserialized = serde_json::from_value::<AddRemoveField>(json!({
            "name": "test_field",
            "type": "string",
            "nullable": true,
        }))
        .unwrap();

        assert_eq!(Value::Null, deserialized.default);
    }
}
