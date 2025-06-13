use jtd::{FromSerdeSchemaError, Schema, SerdeSchema};
use patch::{KeyOrIndex, Path};
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
            Lens::In(in_) => Lens::In(In {
                name: in_.name.clone(),
                ops: in_.ops.iter().rev().map(|lens| lens.reversed()).collect(),
            }),
            Lens::Map(map) => Lens::Map(Map {
                ops: map.ops.iter().rev().map(|lens| lens.reversed()).collect(),
            }),
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

    pub fn transform_jtd(&self, schema: &mut jtd::Schema) -> Result<(), TransformJtdError> {
        // First, check to see if the schema is empty. In that case, we
        // assume it's an object (`properties`) and restart.
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
                properties_is_present: true,
                additional_properties: false,
            };
        }

        // Next, we transform!
        match (self, schema) {
            (Lens::Add(add_remove), Schema::Properties { properties, .. }) => {
                if properties.contains_key(&add_remove.name) {
                    return Err(TransformJtdError::KeyConflict(add_remove.name.clone()));
                }

                properties.insert(
                    add_remove.name.clone(),
                    Schema::from_serde_schema(add_remove.type_.clone())?,
                );

                Ok(())
            }

            (Lens::Remove(add_remove), Schema::Properties { properties, .. }) => {
                properties.remove(&add_remove.name).ok_or_else(|| {
                    TransformJtdError::MissingName(self.name(), add_remove.name.clone())
                })?;

                Ok(())
            }

            (Lens::Rename(Rename { from, to }), Schema::Properties { properties, .. }) => {
                let existing = properties
                    .remove(from)
                    .ok_or_else(|| TransformJtdError::MissingName(self.name(), from.clone()))?;

                properties.insert(to.clone(), existing);

                Ok(())
            }

            (Lens::Extract(extract_embed), Schema::Properties { properties, .. }) => {
                let (host, mut existing) = properties
                    .remove_entry(&extract_embed.host)
                    .ok_or_else(|| {
                        TransformJtdError::MissingName(self.name(), extract_embed.host.clone())
                    })?;

                if let Schema::Properties {
                    properties: host_props,
                    ..
                } = &mut existing
                {
                    if let Some(definition) = host_props.remove(&extract_embed.name) {
                        properties.insert(host, definition);

                        Ok(())
                    } else {
                        properties.insert(host.clone(), existing); // replace so we don't mangle input object

                        Err(TransformJtdError::Within(
                            extract_embed.host.clone(),
                            Box::new(TransformJtdError::MissingName(
                                self.name(),
                                extract_embed.name.clone(),
                            )),
                        ))
                    }
                } else {
                    let err = TransformJtdError::ExpectedXGotY(
                        self.name(),
                        "properties",
                        schema_name(&existing),
                    );

                    properties.insert(host, existing); // replace so we don't mangle input object

                    Err(err)
                }
            }

            (Lens::Embed(extract_embed), Schema::Properties { properties, .. }) => {
                if let Some((host, definition)) = properties.remove_entry(&extract_embed.host) {
                    properties.insert(
                        host,
                        Schema::Properties {
                            definitions: BTreeMap::new(),
                            metadata: BTreeMap::new(),
                            nullable: false,
                            properties: BTreeMap::from([(extract_embed.name.clone(), definition)]),
                            additional_properties: false,
                            optional_properties: BTreeMap::new(),
                            properties_is_present: true,
                        },
                    );

                    Ok(())
                } else {
                    Err(TransformJtdError::MissingName(
                        self.name(),
                        extract_embed.host.clone(),
                    ))
                }
            }

            (Lens::Head(WrapHead { name }), Schema::Properties { properties, .. }) => {
                let (host, existing) = properties
                    .remove_entry(name)
                    .ok_or_else(|| TransformJtdError::MissingName(self.name(), name.clone()))?;

                if let Schema::Elements { elements, .. } = existing {
                    properties.insert(host, *elements);

                    Ok(())
                } else {
                    let err = TransformJtdError::ExpectedXGotY(
                        self.name(),
                        "elements",
                        schema_name(&existing),
                    );

                    properties.insert(host, existing); // replace so we don't mangle input object

                    Err(err)
                }
            }

            (Lens::Wrap(WrapHead { name }), Schema::Properties { properties, .. }) => {
                if let Some((host, definition)) = properties.remove_entry(name) {
                    properties.insert(
                        host,
                        Schema::Elements {
                            definitions: BTreeMap::new(),
                            metadata: BTreeMap::new(),
                            nullable: false,
                            elements: Box::new(definition),
                        },
                    );

                    Ok(())
                } else {
                    Err(TransformJtdError::MissingName(self.name(), name.clone()))
                }
            }

            (Lens::In(in_), Schema::Properties { properties, .. }) => {
                if let Some(prop) = properties.get_mut(&in_.name) {
                    for op in &in_.ops {
                        op.transform_jtd(prop).map_err(|err| {
                            TransformJtdError::Within(in_.name.clone(), Box::new(err))
                        })?;
                    }

                    Ok(())
                } else {
                    Err(TransformJtdError::MissingName(
                        self.name(),
                        in_.name.clone(),
                    ))
                }
            }

            (Lens::Map(map), Schema::Elements { elements, .. }) => {
                for op in &map.ops {
                    op.transform_jtd(elements)?
                }

                Ok(())
            }
            (Lens::Map(_), schema) => Err(TransformJtdError::ExpectedXGotY(
                "map",
                "elements",
                schema_name(schema),
            )),

            (Lens::Convert(convert), Schema::Properties { properties, .. }) => {
                if let Some(prop) = properties.get_mut(&convert.name) {
                    let expected_type = Schema::from_serde_schema(convert.from_type.clone())?;

                    if prop == &expected_type {
                        *prop = Schema::from_serde_schema(convert.to_type.clone())?;

                        Ok(())
                    } else {
                        Err(TransformJtdError::WrongTypeForTransform(
                            Box::new(expected_type),
                            Box::new(prop.clone()),
                        ))
                    }
                } else {
                    Err(TransformJtdError::MissingName(
                        self.name(),
                        convert.name.clone(),
                    ))
                }
            }

            (_, schema) => Err(TransformJtdError::ExpectedXGotY(
                self.name(),
                "properties",
                schema_name(schema),
            )),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            Lens::Add(..) => "add",
            Lens::Remove(..) => "remove",
            Lens::Rename(..) => "rename",
            Lens::Extract(..) => "extract",
            Lens::Embed(..) => "embed",
            Lens::Head(..) => "head",
            Lens::Wrap(..) => "wrap",
            Lens::In(..) => "in",
            Lens::Map(..) => "map",
            Lens::Convert(..) => "convert",
        }
    }

    /// Transform the path of a patch.
    #[must_use]
    pub fn transform_path(&self, path: &mut Path) -> PathMeta {
        match self {
            Lens::Add(..) => PathMeta::Keep,
            Lens::Remove(AddRemove { name, .. }) => match path.first() {
                Some(KeyOrIndex::Key(segment)) if segment == name => PathMeta::Remove,
                _ => PathMeta::Keep,
            },
            Lens::Rename(Rename { from, to }) => {
                if matches!(path.first(), Some(KeyOrIndex::Key(segment)) if segment == from) {
                    path[0] = KeyOrIndex::Key(to.clone());
                }

                PathMeta::Keep
            }
            Lens::Extract(ExtractEmbed { host, name }) => {
                match path.get(0) {
                    Some(KeyOrIndex::Key(host_segment)) if host_segment == host => {
                        match path.get(1) {
                            Some(KeyOrIndex::Key(name_segment)) if name_segment == name => {
                                path.remove(1);

                                PathMeta::Keep
                            }
                            // If the host matched but the name didn't, we're
                            // extracting out of the object anyway and should
                            // drop other keys.
                            _ => PathMeta::Remove,
                        }
                    }
                    _ => PathMeta::Keep,
                }
            }
            Lens::Embed(ExtractEmbed { host, name }) => {
                if matches!(path.get(0), Some(KeyOrIndex::Key(host_segment)) if host_segment == host)
                {
                    path.insert(1, name.into());

                    PathMeta::KeepAndAddHost(Path::from([path[0].clone()]))
                } else {
                    PathMeta::Keep
                }
            }
            Lens::Head(..) => todo!("Lens::Head"),
            Lens::Wrap(..) => todo!("Lens::Wrap"),
            Lens::In(..) => todo!("Lens::In"),
            Lens::Map(..) => todo!("Lens::Map"),
            Lens::Convert(..) => todo!("Lens::Convert"),
        }
    }

    pub fn transform_value(&self, _value: &mut Value) {
        todo!()
    }
}

fn schema_name(schema: &jtd::Schema) -> &'static str {
    match schema {
        Schema::Empty { .. } => "empty",
        Schema::Ref { .. } => "ref",
        Schema::Type { .. } => "type",
        Schema::Enum { .. } => "enum",
        Schema::Elements { .. } => "elements",
        Schema::Properties { .. } => "properties",
        Schema::Values { .. } => "values",
        Schema::Discriminator { .. } => "discriminator",
    }
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum TransformJtdError {
    #[error("Problem with type schema: {0}")]
    ConversionFromSerde(#[from] FromSerdeSchemaError),

    #[error("`{0}` lens expected `{0}`, but got a `{1}` instead.")]
    ExpectedXGotY(&'static str, &'static str, &'static str),

    #[error("`{0}` lens expected a name `{1}`, but it was not present in the properties.")]
    MissingName(&'static str, String),

    #[error("In `{0}`: {1}")]
    Within(String, Box<TransformJtdError>),

    #[error("Could not add key {0}, it already exists in the properties")]
    KeyConflict(String),

    #[error("Got the wrong source type for `transform`. Expected `{0:?}`, got `{1:?}`.")]
    WrongTypeForTransform(Box<Schema>, Box<Schema>),
}

#[derive(Debug, PartialEq)]
pub enum PathMeta {
    Keep,
    KeepAndAddHost(Path),
    Remove,
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
                Err(TransformJtdError::MissingName("rename", "old".to_string()))
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

            assert_eq!(
                result,
                Err(TransformJtdError::KeyConflict("new".to_string()))
            );
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
                Err(TransformJtdError::MissingName(
                    "remove",
                    "missing".to_string()
                ))
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
                Err(TransformJtdError::MissingName(
                    "extract",
                    "user".to_string()
                )),
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
                Err(TransformJtdError::Within(
                    "user".to_string(),
                    Box::new(TransformJtdError::MissingName("extract", "id".to_string())),
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

            assert_eq!(
                Err(TransformJtdError::ExpectedXGotY(
                    "extract",
                    "properties",
                    "type"
                )),
                result
            );
        }

        #[test]
        fn embed_ok() {
            let lens = lens!({
                "embed": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": { "type": "string" }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema!({
                    "properties": {
                        "user": {
                            "properties": {
                                "id": { "type": "string" }
                            }
                        }
                    }
                }),
                schema,
            );
        }

        #[test]
        fn test_embed_missing_host() {
            let lens = lens!({
                "embed": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": { "type": "string" }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema!({
                    "properties": {
                        "user": {
                            "properties": {
                                "id": { "type": "string" }
                            }
                        }
                    }
                }),
                schema,
            );
        }

        #[test]
        fn test_embed_missing_name() {
            let lens = lens!({
                "embed": {
                    "host": "user",
                    "name": "id",
                }
            });

            let mut schema = schema!({ "properties": { } });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::MissingName("embed", "user".into()))
            );
        }

        #[test]
        fn head_ok() {
            let lens = lens!({
                "head": {
                    "name": "items",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "items": {
                        "elements": {
                            "type": "string"
                        }
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "items": {
                            "type": "string"
                        }
                    }
                })
            );
        }

        #[test]
        fn head_missing_name() {
            let lens = lens!({
                "head": {
                    "name": "items",
                }
            });

            let mut schema = schema!({ "properties": {} });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::MissingName("head", "items".to_string()))
            );
        }

        #[test]
        fn head_wrong_type() {
            let lens = lens!({
                "head": {
                    "name": "items",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "items": {
                        "type": "string"
                    }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::ExpectedXGotY("head", "elements", "type"))
            );
        }

        #[test]
        fn wrap_ok() {
            let lens = lens!({
                "wrap": {
                    "name": "items",
                }
            });

            let mut schema = schema!({
                "properties": {
                    "items": {
                        "type": "string"
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "items": {
                            "elements": {
                                "type": "string"
                            }
                        }
                    }
                })
            );
        }

        #[test]
        fn wrap_missing_name() {
            let lens = lens!({
                "wrap": {
                    "name": "items",
                }
            });

            let mut schema = schema!({ "properties": {} });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::MissingName("wrap", "items".to_string()))
            );
        }

        #[test]
        fn in_ok() {
            let lens = lens!({
                "in": {
                    "name": "user",
                    "ops": [{
                        "rename": {
                            "from": "old_field",
                            "to": "new_field"
                        }
                    }]
                }
            });

            let mut schema = schema!({
                "properties": {
                    "user": {
                        "properties": {
                            "old_field": {
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
                        "user": {
                            "properties": {
                                "new_field": {
                                    "type": "string"
                                }
                            }
                        }
                    }
                })
            );
        }

        #[test]
        fn in_missing_name() {
            let lens = lens!({
                "in": {
                    "name": "user",
                    "ops": [{
                        "rename": {
                            "from": "old_field",
                            "to": "new_field"
                        }
                    }]
                }
            });

            let mut schema = schema!({ "properties": {} });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::MissingName("in", "user".to_string()))
            );
        }

        #[test]
        fn in_problem_with_inner_lens() {
            let lens = lens!({
                "in": {
                    "name": "user",
                    "ops": [{
                        "rename": {
                            "from": "nonexistent_field",
                            "to": "new_field"
                        }
                    }]
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
                result,
                Err(TransformJtdError::Within(
                    "user".to_string(),
                    Box::new(TransformJtdError::MissingName(
                        "rename",
                        "nonexistent_field".to_string()
                    ))
                ))
            );
        }

        #[test]
        fn map_ok() {
            let lens = lens!({
                "map": {
                    "ops": [{
                        "rename": {
                            "from": "old_field",
                            "to": "new_field"
                        }
                    }]
                }
            });

            let mut schema = schema!({
                "elements": {
                    "properties": {
                        "old_field": {
                            "type": "string"
                        }
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "elements": {
                        "properties": {
                            "new_field": {
                                "type": "string"
                            }
                        }
                    }
                })
            );
        }

        #[test]
        fn map_wrong_type() {
            let lens = lens!({
                "map": {
                    "ops": [{
                        "rename": {
                            "from": "old_field",
                            "to": "new_field"
                        }
                    }]
                }
            });

            let mut schema = schema!({
                "properties": {
                    "some_field": {
                        "type": "string"
                    }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            assert_eq!(
                result,
                Err(TransformJtdError::ExpectedXGotY(
                    "map",
                    "elements",
                    "properties"
                ))
            );
        }

        #[test]
        fn convert_ok() {
            let lens = lens!({
                "convert": {
                    "name": "status",
                    "from_type": {"type": "string"},
                    "to_type": {"type": "int32"},
                    "forward": {"active": 1, "inactive": 0},
                    "reverse": {"1": "active", "0": "inactive"}
                }
            });

            let mut schema = schema!({
                "properties": {
                    "status": {
                        "type": "string"
                    }
                }
            });

            lens.transform_jtd(&mut schema).unwrap();

            assert_eq!(
                schema,
                schema!({
                    "properties": {
                        "status": {
                            "type": "int32"
                        }
                    }
                })
            );
        }

        #[test]
        fn convert_wrong_type() {
            let lens = lens!({
                "convert": {
                    "name": "status",
                    "from_type": {"type": "string"},
                    "to_type": {"type": "int32"},
                    "forward": {"active": 1, "inactive": 0},
                    "reverse": {"1": "active", "0": "inactive"}
                }
            });

            let mut schema = schema!({
                "properties": {
                    "status": {
                        "type": "int32"
                    }
                }
            });

            let result = lens.transform_jtd(&mut schema);

            let expected_type = schema!({"type": "string"});
            let actual_type = schema!({"type": "int32"});

            assert_eq!(
                result,
                Err(TransformJtdError::WrongTypeForTransform(
                    Box::new(expected_type),
                    Box::new(actual_type)
                ))
            );
        }
    }

    mod transform_path {
        use super::*;
        use pretty_assertions::assert_eq;

        macro_rules! assert_no_op {
            ($lens:expr, $path:expr) => {
                let mut orig = $path;
                let unchanged = orig.clone();
                let meta = $lens.transform_path(&mut orig);
                assert_eq!(PathMeta::Keep, meta);
                assert_eq!(unchanged, orig);
            };
        }

        #[test]
        fn add_always_keeps() {
            let lens = lens!({
                "add": {
                    "name": "new",
                    "type": {"type": "string"}
                }
            });

            assert_no_op!(lens, Path::new());
        }

        #[test]
        fn remove_keeps_if_path_not_same() {
            let lens = lens!({
                "remove": {
                    "name": "name",
                    "type": {"type": "string"}
                }
            });

            assert_no_op!(lens, Path::new());
        }

        #[test]
        fn remove_removes_if_path_matches() {
            let lens = lens!({
                "remove": {
                    "name": "name",
                    "type": {"type": "string"}
                }
            });

            let meta = lens.transform_path(&mut Path::from(["name".into()]));
            assert_eq!(PathMeta::Remove, meta);
        }

        #[test]
        fn rename_skips_if_path_not_same() {
            let lens = lens!({
                "rename": {
                    "from": "old",
                    "to": "new"
                }
            });

            assert_no_op!(lens, Path::from(["whatever".into()]));
        }

        #[test]
        fn rename_renames_if_path_matches() {
            let lens = lens!({
                "rename": {
                    "from": "old",
                    "to": "new"
                }
            });

            let mut path = Path::from(["old".into()]);

            assert_eq!(PathMeta::Keep, lens.transform_path(&mut path));
            assert_eq!(path, Path::from(["new".into()]));
        }

        #[test]
        fn extract_skips_if_host_is_wrong() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id"
                }
            });

            assert_no_op!(lens, Path::from(["other_host".into(), "id".into()]));
        }

        #[test]
        fn extract_removes_other_names_within_host() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id"
                }
            });

            let meta = lens.transform_path(&mut Path::from(["user".into(), "other_name".into()]));

            assert_eq!(PathMeta::Remove, meta);
        }

        #[test]
        fn extract_extracts_if_both_match() {
            let lens = lens!({
                "extract": {
                    "host": "user",
                    "name": "id"
                }
            });

            let mut path = Path::from(["user".into(), "id".into()]);

            assert_eq!(PathMeta::Keep, lens.transform_path(&mut path));
            assert_eq!(path, Path::from(["user".into()]));
        }

        #[test]
        fn embed_skips_if_host_is_wrong() {
            let lens = lens!({
                "embed": {
                    "host": "user",
                    "name": "id"
                }
            });

            assert_no_op!(lens, Path::from(["other_host".into()]));
        }

        #[test]
        fn embed_embeds_if_host_matches() {
            let lens = lens!({
                "embed": {
                    "host": "user",
                    "name": "id"
                }
            });

            let mut path = Path::from(["user".into()]);

            assert_eq!(
                PathMeta::KeepAndAddHost(path.clone()),
                lens.transform_path(&mut path)
            );
            assert_eq!(path, Path::from(["user".into(), "id".into()]));
        }
    }
}
