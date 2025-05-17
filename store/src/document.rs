use crate::operation::{AssignKey, Operation};
use crate::timestamp::Timestamp;
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Debug)]
pub struct Document<Val: Ord> {
    operations: BTreeSet<(Timestamp, Operation<Val>)>,
}

impl<Val: Ord> Document<Val> {
    pub fn new() -> Self {
        Self {
            operations: BTreeSet::new(),
        }
    }

    fn next_timestamp_counter(&self) -> u64 {
        self.operations
            .iter()
            .max()
            .map(|ts| ts.0.counter + 1)
            .unwrap_or(0)
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
