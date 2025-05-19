mod list;
mod map;
mod object;
mod operation;

pub use operation::AssignKey;

use crate::timestamp::Timestamp;
use list::List;
use map::Map;
use object::Object;
use std::collections::{HashMap, HashSet};
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

        self.objects.insert(id, Object::Map(Map::new(id)));

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
        _obj: Timestamp,
        _key: AssignKey,
        _val: Timestamp,
        _prev: HashSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: look up key, insert into map or list, fail on val or missing key
        // self.operations.insert((
        //     id,
        //     Operation::Assign {
        //         obj,
        //         key,
        //         val,
        //         prev,
        //     },
        // ));

        id
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
        obj: Timestamp,
        key: AssignKey,
        prev: HashSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: look up key, insert into map or list, fail on val or missing key
        // self.operations
        //     .insert((id, Operation::Remove { obj, key, prev }));

        id
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
}
