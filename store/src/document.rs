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

    pub fn root<'doc>(&'doc mut self) -> Option<Object<'doc, Val>> {
        for (id, op) in &self.operations {
            match op {
                Operation::MakeMap => {
                    return Some(Object::Map(Map {
                        subject: *id,
                        document: self,
                    }))
                }
                Operation::MakeList => todo!("what if the root object is a list?"),
                Operation::MakeVal { .. } => todo!("what if the root object is a val?"),
                _ => continue,
            }
        }

        None
    }

    pub fn make_map<'doc>(&'doc mut self, node: Uuid) -> Map<'doc, Val> {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((id, Operation::MakeMap));

        Map {
            subject: id,
            document: self,
        }
    }

    fn make_val(&mut self, val: Val, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((id, Operation::MakeVal { val }));

        id
    }

    fn make_assign(
        &mut self,
        obj: Timestamp,
        key: &str,
        val: Timestamp,
        prev: BTreeSet<Timestamp>,
        node: Uuid,
    ) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        self.operations.insert((
            id,
            Operation::Assign {
                obj,
                key: AssignKey::ObjectKey(key.to_string()),
                val,
                prev,
            },
        ));

        id
    }
}

#[derive(Debug)]
pub enum Object<'doc, Val: Ord> {
    Map(Map<'doc, Val>),
}

#[derive(Debug)]
pub struct Map<'doc, Val: Ord> {
    subject: Timestamp,
    document: &'doc mut Document<Val>,
}

impl<'doc, Val: Ord> Map<'doc, Val> {
    pub fn set(&mut self, key: &str, val: Val, node: Uuid) {
        let val_id = self.document.make_val(val, node);
        self.document.make_assign(
            self.subject,
            key,
            val_id,
            // TODO: get the current assignments of this key
            BTreeSet::new(),
            node,
        );
    }
}
