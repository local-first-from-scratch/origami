use crate::type_::SerdeType;
use crate::{Type, Value, value};
use jtd::Schema;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

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

    pub fn transform_defaults(&self, defaults: &mut BTreeMap<String, Value>) -> Result<(), Error> {
        match self {
            Lens::Add(lens) => match defaults.entry(lens.name.clone()) {
                Entry::Vacant(vacant_entry) => {
                    vacant_entry.insert(lens.default.clone());
                    Ok(())
                }
                Entry::Occupied(_) => Err(Error::ConflictingFieldOnAdd(lens.name.clone())),
            },
            Lens::Remove(lens) => match defaults.remove(&lens.name) {
                Some(_) => Ok(()),
                None => Err(Error::MissingFieldOnRemove(lens.name.clone())),
            },
            Lens::Rename { from, to } => match defaults.remove(from) {
                Some(value) => {
                    defaults.insert(to.clone(), value);
                    Ok(())
                }
                None => Err(Error::MissingFieldOnRename(from.clone())),
            },
        }
    }

    pub fn transform_jtd(&self, jtd: &mut Schema) -> Result<(), Error> {
        match jtd {
            Schema::Properties { properties, .. } => match self {
                Lens::Add(lens) => {
                    if properties.contains_key(&lens.name) {
                        Err(Error::ConflictingFieldOnAdd(lens.name.clone()))
                    } else {
                        properties.insert(lens.name.clone(), (&lens.type_).into());
                        Ok(())
                    }
                }
                Lens::Remove(lens) => {
                    if properties.contains_key(&lens.name) {
                        properties.remove(&lens.name);
                        Ok(())
                    } else {
                        Err(Error::MissingFieldOnRemove(lens.name.clone()))
                    }
                }
                Lens::Rename { from, to } => match properties.remove(from) {
                    Some(value) => {
                        properties.insert(to.clone(), value);
                        Ok(())
                    }
                    None => Err(Error::MissingFieldOnRename(from.clone())),
                },
            },

            _ => Err(Error::UnsupportedSchemaType),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("Unsupported schema type. We can only transform `properties` schemas.")]
    UnsupportedSchemaType,
    #[error("Tried to add `{0}`, but it already exists.")]
    ConflictingFieldOnAdd(String),
    #[error("Tried to remove `{0}`, but it was not present.")]
    MissingFieldOnRemove(String),
    #[error("Tried to rename `{0}`, but it was not present.")]
    MissingFieldOnRename(String),
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

impl serde::Serialize for AddRemoveField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let converted = SerdeAddRemoveField {
            name: self.name.clone(),
            type_: self.type_.to_serde(),
            nullable: self.type_.is_nullable(),
            default: self.default.clone(),
        };

        converted.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_::SerdeType;
    use crate::value::Value;
    use serde_json::json;

    mod transform_defaults {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn add_field_success() {
            let mut defaults = BTreeMap::new();

            let lens = Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "default_value".into(),
            });

            lens.transform_defaults(&mut defaults).unwrap();

            let mut expected = BTreeMap::new();
            expected.insert("test".to_string(), "default_value".into());
            assert_eq!(defaults, expected);
        }

        #[test]
        fn add_field_conflict() {
            let mut defaults = BTreeMap::new();
            defaults.insert("test".to_string(), "existing_value".into());

            let lens = Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "default_value".into(),
            });

            assert_eq!(
                lens.transform_defaults(&mut defaults).unwrap_err(),
                Error::ConflictingFieldOnAdd("test".to_string())
            );
        }

        #[test]
        fn remove_field_success() {
            let mut defaults = BTreeMap::new();
            defaults.insert("test".to_string(), "existing_value".into());

            let lens = Lens::Remove(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            lens.transform_defaults(&mut defaults).unwrap();

            assert_eq!(defaults, BTreeMap::new());
        }

        #[test]
        fn remove_field_missing() {
            let mut defaults = BTreeMap::new();

            let lens = Lens::Remove(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            assert_eq!(
                lens.transform_defaults(&mut defaults).unwrap_err(),
                Error::MissingFieldOnRemove("test".to_string())
            );
        }

        #[test]
        fn rename_field_success() {
            let mut defaults = BTreeMap::new();
            defaults.insert("test".to_string(), "value".into());

            let lens = Lens::Rename {
                from: "test".into(),
                to: "new".into(),
            };

            lens.transform_defaults(&mut defaults).unwrap();

            let mut expected = BTreeMap::new();
            expected.insert("new".to_string(), "value".into());
            assert_eq!(defaults, expected);
        }

        #[test]
        fn rename_field_missing() {
            let mut defaults = BTreeMap::new();

            let lens = Lens::Rename {
                from: "test".into(),
                to: "new".into(),
            };

            assert_eq!(
                lens.transform_defaults(&mut defaults).unwrap_err(),
                Error::MissingFieldOnRename("test".to_string())
            );
        }
    }

    mod transform_jtd {
        use super::*;
        use pretty_assertions::assert_eq;

        macro_rules! schema {
            ($properties:tt) => {
                jtd::Schema::from_serde_schema(
                    serde_json::from_value::<jtd::SerdeSchema>(serde_json::json!({"properties": $properties})).unwrap(),
                )
                .unwrap()
            };
        }

        #[test]
        fn add_field_success() {
            let mut base = schema!({});

            let lens = Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            lens.transform_jtd(&mut base).unwrap();

            assert_eq!(base, schema!({"test": {"type": "string"}}));
        }

        #[test]
        fn add_field_conflict() {
            let mut base = schema!({"test": {"type": "string"}});

            let lens = Lens::Add(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            assert_eq!(
                lens.transform_jtd(&mut base).unwrap_err(),
                Error::ConflictingFieldOnAdd("test".to_string())
            );
        }

        #[test]
        fn remove_field_success() {
            let mut base = schema!({"test": {"type": "string"}});

            let lens = Lens::Remove(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            lens.transform_jtd(&mut base).unwrap();

            assert_eq!(base, schema!({}));
        }

        #[test]
        fn remove_field_missing() {
            let mut base = schema!({});

            let lens = Lens::Remove(AddRemoveField {
                name: "test".into(),
                type_: Type::String,
                default: "unused".into(),
            });

            assert_eq!(
                lens.transform_jtd(&mut base).unwrap_err(),
                Error::MissingFieldOnRemove("test".to_string())
            );
        }

        #[test]
        fn rename_field_success() {
            let mut base = schema!({"test": {"type": "string"}});

            let lens = Lens::Rename {
                from: "test".into(),
                to: "new".into(),
            };

            lens.transform_jtd(&mut base).unwrap();

            assert_eq!(base, schema!({"new": {"type": "string"}}));
        }

        #[test]
        fn rename_field_missing() {
            let mut base = schema!({});

            let lens = Lens::Rename {
                from: "test".into(),
                to: "new".into(),
            };

            assert_eq!(
                lens.transform_jtd(&mut base).unwrap_err(),
                Error::MissingFieldOnRename("test".to_string())
            );
        }
    }

    mod deserialize {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn conversion_deserializes_successfully() {
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
        fn conversion_checks_default() {
            let err = serde_json::from_value::<AddRemoveField>(json!({
                "name": "test_field",
                "type": "string",
                "nullable": true,
                "default": 1,
            }))
            .unwrap_err();

            assert_eq!("Invalid value for type nullable string: 1", err.to_string(),);
        }

        #[test]
        fn conversion_sets_null_as_default() {
            let deserialized = serde_json::from_value::<AddRemoveField>(json!({
                "name": "test_field",
                "type": "string",
                "nullable": true,
            }))
            .unwrap();

            assert_eq!(Value::Null, deserialized.default);
        }
    }
}
