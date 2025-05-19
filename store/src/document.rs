mod assign;
mod list;
mod map;
mod object;
mod operation;

pub use operation::AssignKey;
use wasm_bindgen::JsValue;

use crate::timestamp::Timestamp;
use list::List;
use map::Map;
use object::Object;
use std::collections::{BTreeSet, HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug)]
pub struct Document<Val: Ord> {
    objects: HashMap<Timestamp, Object<Val>>,
    highest_counter: u64,
}

impl<Val: Ord> Document<Val> {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            highest_counter: 0,
        }
    }

    fn next_timestamp_counter(&mut self) -> u64 {
        // note for later: we'll have to do a `max` between this and any
        // incoming operations. The increment will be the same, though.
        self.highest_counter += 1;

        self.highest_counter
    }

    pub fn root(&mut self) -> Option<Timestamp> {
        self.objects.keys().min().copied()
    }

    pub fn make_map(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        debug_assert!(
            !self.objects.contains_key(&id),
            "document already contains {id}"
        );

        self.objects
            .insert(id, Object::Map(Map::new_from_root_id(id)));

        id
    }

    pub fn make_val(&mut self, val: Val, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        debug_assert!(
            !self.objects.contains_key(&id),
            "document already contains {id}"
        );

        self.objects.insert(id, Object::Val(id, val));

        id
    }

    pub fn make_list(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        debug_assert!(
            !self.objects.contains_key(&id),
            "document already contains {id}"
        );

        self.objects.insert(id, Object::List(List::new(id)));

        id
    }

    pub fn assign(
        &mut self,
        obj: Timestamp,
        key: AssignKey,
        val: Timestamp,
        prev: BTreeSet<Timestamp>,
        node: Uuid,
    ) -> Result<Timestamp, AssignError> {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        match self.objects.get_mut(&obj) {
            None => Err(AssignError::KeyNotFound),
            Some(Object::Val(..)) => Err(AssignError::ObjectWasVal),
            Some(Object::Map(map)) => {
                if !matches!(key, AssignKey::MapKey(..)) {
                    Err(AssignError::ExpectedMapKey)
                } else {
                    map.assign(id, obj, key, val, prev);
                    Ok(id)
                }
            }
            Some(Object::List(list)) => todo!("list"),
        }
    }

    pub fn insert_after(&mut self, _prev: Timestamp, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: look up key, insert into list, fail on map, val, or missing key
        // self.operations
        //     .insert((id, Operation::InsertAfter { prev }));

        id
    }

    pub fn remove(
        &mut self,
        _obj: Timestamp,
        _key: AssignKey,
        _prev: BTreeSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: look up key, insert into map or list, fail on val or missing key
        // self.operations
        //     .insert((id, Operation::Remove { obj, key, prev }));

        id
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum AssignError {
    #[error("Object not found.")]
    KeyNotFound,
    #[error("Object was found, but was a val. Only maps and lists can have assignments.")]
    ObjectWasVal,
    #[error("Expected a map key, but got an insert after.")]
    ExpectedMapKey,
}

impl Into<JsValue> for AssignError {
    fn into(self) -> JsValue {
        JsValue::from_str(&self.to_string())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn make_map_gives_timestamp_for_map() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        let map_id = doc.make_map(node_id);

        // The timestamp should now exist in the document
        assert!(doc.objects.contains_key(&map_id));

        // The object should be a Map
        match &doc.objects[&map_id] {
            Object::Map(_) => {}
            _ => panic!("Expected Map object for timestamp {}", map_id),
        }
    }

    #[test]
    fn make_list_gives_timestamp_for_list() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        let list_id = doc.make_list(node_id);

        // The timestamp should now exist in the document
        assert!(doc.objects.contains_key(&list_id));

        // The object should be a List
        match &doc.objects[&list_id] {
            Object::List(_) => {}
            _ => panic!("Expected List object for timestamp {}", list_id),
        }
    }

    #[test]
    fn make_val_gives_timestamp_for_val() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();
        let value = 42;

        let val_id = doc.make_val(value, node_id);

        // The timestamp should now exist in the document
        assert!(doc.objects.contains_key(&val_id));

        // The object should be the correct value
        match &doc.objects[&val_id] {
            Object::Val(_, v) => {
                assert_eq!(*v, value);
            }
            _ => panic!("Expected Val object for timestamp {}", val_id),
        }
    }

    #[test]
    fn assign_to_non_existent_object_gives_key_not_found() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        // Create a timestamp that doesn't exist in the document
        let non_existent_id = Timestamp::new(999, node_id);

        // Create a value to assign
        let val_id = doc.make_val(42, node_id);

        // Try to assign the value to a non-existent object
        let result = doc.assign(
            non_existent_id,
            AssignKey::MapKey("key".to_string()),
            val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that we get the KeyNotFound error
        assert_eq!(result, Err(AssignError::KeyNotFound));
    }

    #[test]
    fn assign_to_val_object_gives_object_was_val() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        // Create a value object
        let val_id = doc.make_val(42, node_id);

        // Create another value to assign
        let another_val_id = doc.make_val(99, node_id);

        // Try to assign the second value to the first value (which should fail because we can only assign to maps or lists)
        let result = doc.assign(
            val_id,
            AssignKey::MapKey("key".to_string()),
            another_val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that we get the ObjectWasVal error
        assert_eq!(result, Err(AssignError::ObjectWasVal));
    }

    #[test]
    fn assign_to_map_with_insert_after_gives_expected_map_key() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        // Create a map
        let map_id = doc.make_map(node_id);

        // Create a value to assign
        let val_id = doc.make_val(42, node_id);

        // Create another timestamp to use as the "prev" in the InsertAfter
        let prev_id = doc.make_val(99, node_id);

        // Try to assign to the map using InsertAfter instead of MapKey
        let result = doc.assign(
            map_id,
            AssignKey::InsertAfter(prev_id),
            val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that we get the ExpectedMapKey error
        assert_eq!(result, Err(AssignError::ExpectedMapKey));
    }
}
