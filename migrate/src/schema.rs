use crate::{Type, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub type_: Type,
    pub default: Value,
}

#[derive(Debug, Default, PartialEq)]
pub struct Schema(BTreeMap<String, Field>);

impl Schema {
    pub fn contains_key<S: AsRef<str>>(&self, name: S) -> bool {
        self.0.contains_key(name.as_ref())
    }

    pub fn insert(&mut self, name: String, field: Field) -> Option<Field> {
        self.0.insert(name, field)
    }

    pub fn remove<S: AsRef<str>>(&mut self, name: S) -> Option<Field> {
        self.0.remove(name.as_ref())
    }

    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&Field> {
        self.0.get(name.as_ref())
    }
}

impl From<Schema> for jtd::Schema {
    fn from(schema: Schema) -> Self {
        let mut properties = BTreeMap::new();

        for (name, field) in schema.0 {
            properties.insert(name, (&field.type_).into());
        }

        jtd::Schema::Properties {
            definitions: BTreeMap::new(),
            metadata: BTreeMap::new(),
            nullable: false,
            properties,
            optional_properties: BTreeMap::new(),
            properties_is_present: true,
            additional_properties: false,
        }
    }
}

impl<const N: usize> From<[(&str, Field); N]> for Schema {
    fn from(array: [(&str, Field); N]) -> Self {
        let mut map = BTreeMap::new();
        for (name, field) in array {
            map.insert(name.to_string(), field);
        }
        Schema(map)
    }
}
