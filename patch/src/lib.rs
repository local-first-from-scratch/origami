use serde_json::{Value, json};

#[derive(Debug, Clone)]
pub enum KeyOrIndex {
    Key(String),
    LastIndex,
}

pub type Path = Vec<KeyOrIndex>;

#[derive(Debug)]
pub struct SetOp {
    pub path: Path,
    pub value: Value,
    pub schema: String,
}

pub fn to_value(patches: &Vec<SetOp>) -> Result<Value, Error> {
    let mut base = json!({});

    for patch in patches {
        let mut cursor = &mut base;
        for segment in patch.path.iter().take(patch.path.len() - 1) {
            match (segment, cursor) {
                (KeyOrIndex::Key(key), Value::Object(map)) => match map.get_mut(key) {
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not navigate to path {0:?}")]
    CouldNotNavigateToPath(Path),

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
}
