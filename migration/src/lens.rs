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
                Lens::Add(add_remove) => todo!(),
                Lens::Remove(add_remove) => todo!(),
                Lens::Rename(Rename { from, to }) => {
                    let existing = properties
                        .remove(from)
                        .ok_or(ApplyToJtdError::MissingRenameKey(from.clone()))?;

                    properties.insert(to.clone(), existing);
                }
                Lens::Extract(extract_embed) => todo!(),
                Lens::Embed(extract_embed) => todo!(),
                Lens::Head(wrap_head) => todo!(),
                Lens::Wrap(wrap_head) => todo!(),
                Lens::In(_) => todo!(),
                Lens::Map(map) => todo!(),
                Lens::Convert(convert) => todo!(),
            }

            Ok(())
        } else {
            return Err(ApplyToJtdError::NotARecord);
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
}

#[cfg(test)]
mod test {
    use super::*;

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
    }
}
