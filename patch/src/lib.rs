mod path;

pub use path::{KeyOrIndex, Path};
use serde_json::{Value, json};

#[derive(Debug, PartialEq)]
pub struct SetOp {
    pub path: Path,
    pub value: Value,
    pub schema: String,
}

pub fn to_value(patches: &Vec<SetOp>) -> Result<Value, Error> {
    let mut base = json!({});

    for patch in patches {
        let mut cursor = &mut base;

        for segment in patch.path.all_but_last() {
            match (segment, cursor) {
                (KeyOrIndex::Key(key), Value::Object(map)) => match map.get_mut(key) {
                    Some(next) => cursor = next,
                    None => return Err(Error::CouldNotNavigateToPath(patch.path.clone())),
                },
                (KeyOrIndex::Index(index), Value::Array(arr)) => match arr.get_mut(*index) {
                    Some(next) => cursor = next,
                    None => return Err(Error::CouldNotNavigateToPath(patch.path.clone())),
                },
                (KeyOrIndex::LastIndex, Value::Array(arr)) => match arr.last_mut() {
                    Some(next) => cursor = next,
                    None => return Err(Error::CouldNotNavigateToPath(patch.path.clone())),
                },
                // TODO: add more information here
                _ => return Err(Error::WrongType),
            }
        }

        match patch.path.last() {
            Some(last) => match (last, cursor) {
                (KeyOrIndex::Key(key), Value::Object(map)) => {
                    map.insert(key.clone(), patch.value.clone());
                }
                (KeyOrIndex::Index(index), Value::Array(arr)) => {
                    if *index < arr.len() {
                        arr[*index] = patch.value.clone();
                    } else {
                        return Err(Error::ListIndexOutOfBounds);
                    }
                }
                (KeyOrIndex::LastIndex, Value::Array(arr)) => {
                    arr.push(patch.value.clone());
                }
                // TODO: add more information here
                _ => return Err(Error::WrongType),
            },

            // This patch must have been empty, and we should set the base value from it.
            None => base = patch.value.clone(),
        };
    }

    Ok(base)
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("Could not navigate to path {0:?}")]
    CouldNotNavigateToPath(Path),

    #[error("List index was out of bounds")]
    ListIndexOutOfBounds,

    #[error("Got the wrong type when navigating")]
    WrongType,
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn create_object() {
        let patches = vec![SetOp {
            path: Path::new(),
            value: json!({}),
            schema: "test".to_string(),
        }];

        assert_eq!(to_value(&patches).unwrap(), json!({}));
    }

    #[test]
    fn set_value() {
        let patches = vec![SetOp {
            path: Path::from(["key".into()]),
            value: json!(1),
            schema: "test".to_string(),
        }];

        assert_eq!(to_value(&patches).unwrap(), json!({"key": 1}));
    }

    #[test]
    fn append_to_list() {
        let patches = vec![
            SetOp {
                path: Path::from(["list".into()]),
                value: json!([]),
                schema: "test".to_string(),
            },
            SetOp {
                path: Path::from(["list".into(), KeyOrIndex::LastIndex]),
                value: json!(1),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(to_value(&patches).unwrap(), json!({"list": [1]}));
    }

    #[test]
    fn navigate_through_array_index() {
        let patches = vec![
            SetOp {
                path: Path::from(["list".into()]),
                value: json!([{"key": "value"}]),
                schema: "test".to_string(),
            },
            SetOp {
                path: Path::from(["list".into(), 0.into(), "key".into()]),
                value: json!("updated"),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(
            to_value(&patches).unwrap(),
            json!({"list": [{"key": "updated"}]})
        );
    }

    #[test]
    fn set_at_specific_index() {
        let patches = vec![
            SetOp {
                path: Path::from(["list".into()]),
                value: json!([1, 2, 3]),
                schema: "test".to_string(),
            },
            SetOp {
                path: Path::from(["list".into(), 1.into()]),
                value: json!(99),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(to_value(&patches).unwrap(), json!({"list": [1, 99, 3]}));
    }

    #[test]
    fn cannot_navigate_to_out_of_bounds_index() {
        let path = Path::from(["list".into(), 5.into(), "irrelevant".into()]);
        let patches = vec![
            SetOp {
                path: Path::from(["list".into()]),
                value: json!([1, 2]),
                schema: "test".to_string(),
            },
            SetOp {
                path: path.clone(),
                value: json!(99),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(to_value(&patches), Err(Error::CouldNotNavigateToPath(path)));
    }

    #[test]
    fn cannot_set_at_out_of_bounds_index() {
        let path = Path::from(["list".into(), 10.into()]);
        let patches = vec![
            SetOp {
                path: Path::from(["list".into()]),
                value: json!([1, 2, 3]),
                schema: "test".to_string(),
            },
            SetOp {
                path: path.clone(),
                value: json!(99),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(to_value(&patches), Err(Error::ListIndexOutOfBounds));
    }

    #[test]
    fn cannot_navigate_to_string_key() {
        let path = Path::from(["hello".into(), "world".into()]);
        let patches = vec![SetOp {
            path: path.clone(),
            value: json!("hello"),
            schema: "test".to_string(),
        }];

        assert_eq!(to_value(&patches), Err(Error::CouldNotNavigateToPath(path)));
    }

    #[test]
    fn cannot_navigate_through_list() {
        let path = Path::from(["hello".into(), KeyOrIndex::LastIndex, "world".into()]);
        let patches = vec![
            SetOp {
                path: Path::from(["hello".into()]),
                value: json!([]),
                schema: "test".to_string(),
            },
            SetOp {
                path: path.clone(),
                value: json!(1),
                schema: "test".to_string(),
            },
        ];

        assert_eq!(to_value(&patches), Err(Error::CouldNotNavigateToPath(path)));
    }

    #[test]
    fn wrong_type_for_navigation() {
        let patches = vec![
            SetOp {
                path: Path::new(),
                value: json!(1),
                schema: "test".to_string(),
            },
            SetOp {
                path: Path::from(["key".into()]),
                value: json!(1),
                schema: "test".to_string(),
            },
        ];

        assert!(matches!(to_value(&patches), Err(Error::WrongType)));
    }

    #[test]
    fn wrong_type_for_assignment() {
        let patches = vec![
            SetOp {
                path: Path::new(),
                value: json!("hello"),
                schema: "test".to_string(),
            },
            SetOp {
                path: Path::from([KeyOrIndex::LastIndex]),
                value: json!(1),
                schema: "test".to_string(),
            },
        ];

        assert!(matches!(to_value(&patches), Err(Error::WrongType)));
    }
}
