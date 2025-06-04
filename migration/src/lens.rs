use jtd::{FromSerdeSchemaError, Schema, SerdeSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
#[allow(clippy::large_enum_variant)] // TODO: measure memory use and revisit
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
    pub type_: SerdeSchema,
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
    pub from_type: SerdeSchema,
    pub to_type: SerdeSchema,
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

    pub fn transform_jtd(&self, schema: &mut jtd::Schema) -> Result<(), ApplyToJtdError> {
        // First, we convert the schema to an object if it's empty.
        if let Schema::Empty {
            definitions,
            metadata,
        } = schema
        {
            *schema = Schema::Properties {
                definitions: definitions.clone(),
                metadata: metadata.clone(),
                nullable: false,
                properties: BTreeMap::new(),
                optional_properties: BTreeMap::new(),
                properties_is_present: false,
                additional_properties: false,
            };
        }

        // Next we modify!
        if let Schema::Properties { properties, .. } = schema {
            match self {
                Lens::Add(add_remove) => {
                    if properties.contains_key(&add_remove.name) {
                        return Err(ApplyToJtdError::KeyConflict(add_remove.name.clone()));
                    }

                    properties.insert(
                        add_remove.name.clone(),
                        Schema::from_serde_schema(add_remove.type_.clone())?,
                    );
                }
                Lens::Remove(add_remove) => {
                    properties.remove(&add_remove.name).ok_or_else(|| {
                        ApplyToJtdError::MissingRemoveKey(add_remove.name.clone())
                    })?;
                }
                Lens::Rename(Rename { from, to }) => {
                    let existing = properties
                        .remove(from)
                        .ok_or_else(|| ApplyToJtdError::MissingRenameKey(from.clone()))?;

                    properties.insert(to.clone(), existing);
                }
                Lens::Extract(extract_embed) => {
                    let (host, mut existing) = properties
                        .remove_entry(&extract_embed.host)
                        .ok_or_else(|| {
                            ApplyToJtdError::MissingExtractHost(extract_embed.host.clone())
                        })?;

                    if let Schema::Properties {
                        properties: host_props,
                        ..
                    } = &mut existing
                    {
                        if let Some(definition) = host_props.remove(&extract_embed.name) {
                            properties.insert(host, definition);
                        } else {
                            properties.insert(host.clone(), existing); // replace to not mangle object
                            return Err(ApplyToJtdError::MissingExtractName(
                                host,
                                extract_embed.name.clone(),
                            ));
                        }
                    } else {
                        properties.insert(host, existing); // replace to not mangle object
                        return Err(ApplyToJtdError::ExtractExpectedProperties);
                    }
                }
                Lens::Embed(_extract_embed) => todo!(),
                Lens::Head(_wrap_head) => todo!(),
                Lens::Wrap(_wrap_head) => todo!(),
                Lens::In(_) => todo!(),
                Lens::Map(_map) => todo!(),
                Lens::Convert(_convert) => todo!(),
            }

            Ok(())
        } else {
            Err(ApplyToJtdError::NotARecord)
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ApplyToJtdError {
    #[error("Problem with type schema: {0}")]
    ConversionFromSerde(#[from] FromSerdeSchemaError),

    #[error("We can only modify records")]
    NotARecord,

    #[error("Could not rename key {0}, it did not exist in the properties")]
    MissingRenameKey(String),

    #[error("Could not add key {0}, it already exists in the properties")]
    KeyConflict(String),

    #[error("Could not remove key {0}, it did not exist in the properties")]
    MissingRemoveKey(String),

    #[error("Missing extract host {0}")]
    MissingExtractHost(String),

    #[error("Extract expected properties, but got something else")]
    ExtractExpectedProperties,

    #[error("Found host for {0} but not name {1}")]
    MissingExtractName(String, String),
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! lens {
        ($patch:tt) => {
            serde_json::from_value::<Lens>(serde_json::json!($patch)).unwrap()
        };
    }

    macro_rules! schema {
        ($patch:tt) => {
            Schema::from_serde_schema(
                serde_json::from_value::<SerdeSchema>(serde_json::json!($patch)).unwrap(),
            )
            .unwrap()
        };
    }

    mod transform_jtd {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn rename_ok() {
            let lens = Lens::Rename(Rename {
                from: "old".to_string(),
                to: "new".to_string(),
            });

            let mut schema = schema!({
                "properties": {
                    "old": {
                        "type": "string"
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "new": {
                            "type": "string"
                        }
                    }
                })
            );
        }

        #[test]
        fn rename_err() {
            let lens = Lens::Rename(Rename {
                from: "old".to_string(),
                to: "new".to_string(),
            });

            let mut schema = schema!({"properties": {}});

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(ApplyToJtdError::MissingRenameKey("old".to_string()))
            );
        }

        #[test]
        fn add_ok() {
            let lens = lens!({
                "add": {
                    "name": "new",
                    "type": {"type": "string"}
                }
            });

            let mut schema = schema!({"properties": {}});

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "new": {
                            "type": "string"
                        }
                    }
                })
            );
        }

        #[test]
        fn add_conflict() {
            let lens = lens!({
                "add": {
                    "name": "new",
                    "type": {"type": "string"}
                }
            });

            let mut schema = schema!({
                "properties": {
                    "new": {
                        // It doesn't matter if the type is the same, we need to
                        // not add duplicates.
                        "type": "string"
                    }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(result, Err(ApplyToJtdError::KeyConflict("new".to_string())));
        }

        #[test]
        fn remove_ok() {
            let lens = lens!({
                "remove": {
                    "name": "old",
                    "type": { "type": "string" },
                }
            });

            let mut schema = schema!({
                "properties": {
                    "old": {
                        "type": "string"
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(schema, schema!({ "properties": {} }));
        }

        #[test]
        fn remove_missing() {
            let lens = lens!({
                "remove": {
                    "name": "missing",
                    "type": { "type": "string" },
                }
            });

            let mut schema = schema!({ "properties": {} });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(ApplyToJtdError::MissingRemoveKey("missing".to_string()))
            );
        }

        #[test]
        fn extract_ok() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": {
                        "properties": {
                            "id": {
                                "type": "string"
                            }
                        }
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "user": { "type": "string" }
                    }
                })
            );
        }

        #[test]
        fn extract_missing_host() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "irrelevant",
                }
            });

            let mut schema = schema!({ "properties": {} });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                Err(ApplyToJtdError::MissingExtractHost("user".to_string())),
                result,
            );
        }

        #[test]
        fn extract_missing_name() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": {
                        "properties": {}
                    }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                Err(ApplyToJtdError::MissingExtractName(
                    "user".to_string(),
                    "id".to_string()
                )),
                result,
            );
        }

        #[test]
        fn extract_wrong_type() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": { "type": "string" }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(Err(ApplyToJtdError::ExtractExpectedProperties), result,);
        }
    }
}
