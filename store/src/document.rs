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

        // TODO: handle key collision
        self.objects.insert(id, Object::Map(Map::new(id)));

        id
    }

    pub fn make_val(&mut self, val: Val, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: handle key collision
        self.objects.insert(id, Object::Val(id, val));

        id
    }

    pub fn make_list(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);

        // TODO: handle key collision
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
