use std::{collections::BTreeMap, fmt::Display};

use jtd::Schema;

use crate::value::Value;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SerdeType {
    String,
    Int,
    Float,
    Bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Int,
    Float,
    Bool,
    Nullable(Box<Type>),
}

impl Type {
    pub fn from_serde(base: SerdeType, nullable: bool) -> Self {
        if nullable {
            Self::Nullable(Box::new(base.into()))
        } else {
            base.into()
        }
    }

    pub fn to_serde(&self) -> SerdeType {
        match self {
            Type::String => SerdeType::String,
            Type::Int => SerdeType::Int,
            Type::Float => SerdeType::Float,
            Type::Bool => SerdeType::Bool,
            Type::Nullable(inner) => inner.to_serde(),
        }
    }

    pub fn is_nullable(&self) -> bool {
        matches!(self, Type::Nullable(_))
    }

    pub fn validate(&self, value: &Value) -> Result<(), ValidationError> {
        match (self, value) {
            (Type::String, Value::String(_)) => Ok(()),
            (Type::Int, Value::Int(_)) => Ok(()),
            (Type::Float, Value::Float(_)) => Ok(()),
            (Type::Bool, Value::Bool(_)) => Ok(()),
            (Type::Nullable(_), Value::Null) => Ok(()),
            (Type::Nullable(inner), _) => inner.validate(value).map_err(|err| match err {
                ValidationError::InvalidValue { got, .. } => ValidationError::InvalidValue {
                    expected: self.clone(),
                    got,
                },
            }),
            _ => Err(ValidationError::InvalidValue {
                expected: self.clone(),
                got: value.clone(),
            }),
        }
    }
}

impl From<SerdeType> for Type {
    fn from(value: SerdeType) -> Self {
        match value {
            SerdeType::String => Self::String,
            SerdeType::Int => Self::Int,
            SerdeType::Float => Self::Float,
            SerdeType::Bool => Self::Bool,
        }
    }
}

impl From<&Type> for jtd::Type {
    fn from(type_: &Type) -> Self {
        match type_ {
            Type::String => jtd::Type::String,
            Type::Int => jtd::Type::Int32,
            Type::Float => jtd::Type::Float64,
            Type::Bool => jtd::Type::Boolean,
            Type::Nullable(inner) => inner.as_ref().into(),
        }
    }
}

impl From<&Type> for Schema {
    fn from(type_: &Type) -> Self {
        match type_ {
            Type::String | Type::Int | Type::Float | Type::Bool => Schema::Type {
                definitions: BTreeMap::new(),
                metadata: BTreeMap::new(),
                nullable: false,
                type_: type_.into(),
            },
            Type::Nullable(inner) => Schema::Type {
                definitions: BTreeMap::new(),
                metadata: BTreeMap::new(),
                nullable: true,
                type_: inner.as_ref().into(),
            },
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::String => write!(f, "string"),
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::Bool => write!(f, "bool"),
            Type::Nullable(inner) => write!(f, "nullable {}", inner),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid value for type {expected}: {got}")]
    InvalidValue { expected: Type, got: Value },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_string() {
        let string_type = Type::String;
        assert!(
            string_type
                .validate(&Value::String("hello".to_string()))
                .is_ok()
        );
        assert!(string_type.validate(&Value::Int(42)).is_err());
        assert!(string_type.validate(&Value::Float(1.23)).is_err());
        assert!(string_type.validate(&Value::Bool(true)).is_err());
        assert!(string_type.validate(&Value::Null).is_err());
    }

    #[test]
    fn validate_int() {
        let int_type = Type::Int;
        assert!(int_type.validate(&Value::Int(42)).is_ok());
        assert!(
            int_type
                .validate(&Value::String("hello".to_string()))
                .is_err()
        );
        assert!(int_type.validate(&Value::Float(1.23)).is_err());
        assert!(int_type.validate(&Value::Bool(true)).is_err());
        assert!(int_type.validate(&Value::Null).is_err());
    }

    #[test]
    fn validate_float() {
        let float_type = Type::Float;
        assert!(float_type.validate(&Value::Float(1.23)).is_ok());
        assert!(
            float_type
                .validate(&Value::String("hello".to_string()))
                .is_err()
        );
        assert!(float_type.validate(&Value::Int(42)).is_err());
        assert!(float_type.validate(&Value::Bool(true)).is_err());
        assert!(float_type.validate(&Value::Null).is_err());
    }

    #[test]
    fn validate_bool() {
        let bool_type = Type::Bool;
        assert!(bool_type.validate(&Value::Bool(true)).is_ok());
        assert!(bool_type.validate(&Value::Bool(false)).is_ok());
        assert!(
            bool_type
                .validate(&Value::String("hello".to_string()))
                .is_err()
        );
        assert!(bool_type.validate(&Value::Int(42)).is_err());
        assert!(bool_type.validate(&Value::Float(1.23)).is_err());
        assert!(bool_type.validate(&Value::Null).is_err());
    }

    #[test]
    fn validate_nullable() {
        let nullable_string = Type::Nullable(Box::new(Type::String));
        assert!(nullable_string.validate(&Value::Null).is_ok());
        assert!(
            nullable_string
                .validate(&Value::String("hello".to_string()))
                .is_ok()
        );
        assert!(nullable_string.validate(&Value::Int(42)).is_err());

        let nullable_int = Type::Nullable(Box::new(Type::Int));
        assert!(nullable_int.validate(&Value::Null).is_ok());
        assert!(nullable_int.validate(&Value::Int(42)).is_ok());
        assert!(
            nullable_int
                .validate(&Value::String("hello".to_string()))
                .is_err()
        );

        let nullable_float = Type::Nullable(Box::new(Type::Float));
        assert!(nullable_float.validate(&Value::Null).is_ok());
        assert!(nullable_float.validate(&Value::Float(1.23)).is_ok());
        assert!(nullable_float.validate(&Value::Bool(true)).is_err());

        let nullable_bool = Type::Nullable(Box::new(Type::Bool));
        assert!(nullable_bool.validate(&Value::Null).is_ok());
        assert!(nullable_bool.validate(&Value::Bool(true)).is_ok());
        assert!(nullable_bool.validate(&Value::Float(1.23)).is_err());
    }
}
