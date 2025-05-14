use crate::operation::Operation;
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

    pub fn root<'doc>(&'doc self) -> Option<Object<'doc, Val>> {
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
}

#[derive(Debug)]
pub enum Object<'doc, Val: Ord> {
    Map(Map<'doc, Val>),
}

#[derive(Debug)]
pub struct Map<'doc, Val: Ord> {
    subject: Timestamp,
    document: &'doc Document<Val>,
}
