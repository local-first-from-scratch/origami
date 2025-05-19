use super::{AssignKey, operation::Operation};
use crate::timestamp::Timestamp;
use std::collections::{BTreeSet, HashSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct Map<Val: Ord> {
    ops: BTreeSet<(Timestamp, Operation<Val>)>,
}

impl<Val: Ord> Map<Val> {
    pub fn new_from_root_id(root_id: Timestamp) -> Self {
        Self {
            ops: BTreeSet::from([(root_id, Operation::MakeMap)]),
        }
    }

    pub fn assign(
        &mut self,
        id: Timestamp,
        obj: Timestamp,
        key: AssignKey,
        val: Timestamp,
        prev: BTreeSet<Timestamp>,
    ) {
        self.ops.insert((
            id,
            Operation::Assign {
                obj,
                key,
                val,
                prev,
            },
        ));
    }
}
