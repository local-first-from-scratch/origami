mod list;
mod map;
mod object;
mod operation;

pub use operation::AssignKey;

use crate::timestamp::Timestamp;
use operation::Operation;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug)]
pub struct Document<Val: Ord> {
    operations: BTreeSet<(Timestamp, Operation<Val>)>,
    highest_counter: u64,
}

impl<Val: Ord> Document<Val> {
    pub fn new() -> Self {
        Self {
            operations: BTreeSet::new(),
            highest_counter: 0,
        }
    }

    fn next_timestamp_counter(&mut self) -> u64 {
        // note for later: we'll have to do a `max` between this and any
        // incoming operations. The increment will be the same, though.
        self.highest_counter += 1;

        self.highest_counter
    }

    pub fn root<'doc>(&'doc mut self) -> Option<Timestamp> {
        for (id, op) in &self.operations {
            match op {
                Operation::MakeMap | Operation::MakeList => return Some(*id),
                _ => continue,
            }
        }

        None
    }

    pub fn make_map<'doc>(&'doc mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((id, Operation::MakeMap));

        id
    }

    pub fn make_val(&mut self, val: Val, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((id, Operation::MakeVal { val }));

        id
    }

    pub fn make_list<'doc>(&'doc mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((id, Operation::MakeList));

        id
    }

    pub fn assign(
        &mut self,
        obj: Timestamp,
        key: AssignKey,
        val: Timestamp,
        prev: BTreeSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((
            id,
            Operation::Assign {
                obj,
                key,
                val,
                prev,
            },
        ));

        id
    }

    pub fn insert_after(&mut self, prev: Timestamp, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations
            .insert((id, Operation::InsertAfter { prev }));

        id
    }

    pub fn remove(
        &mut self,
        obj: Timestamp,
        key: AssignKey,
        prev: BTreeSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations
            .insert((id, Operation::Remove { obj, key, prev }));

        id
    }
}
