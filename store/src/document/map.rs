use super::AssignKey;
use super::assign::Assign;
use super::operation::Operation;
use crate::timestamp::Timestamp;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Map<Val: Ord> {
    ops: BTreeSet<(Timestamp, Operation<Val>)>,
    current_values: Assign,
}

impl<Val: Ord> Map<Val> {
    pub fn new_from_root_id(root_id: Timestamp) -> Self {
        Self {
            ops: BTreeSet::from([(root_id, Operation::MakeMap)]),
            current_values: Assign::new(),
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
        self.current_values.assign(id, key.clone(), val, &prev);

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
