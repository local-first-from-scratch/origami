use super::operation::Operation;
use crate::timestamp::Timestamp;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct List<Val: Ord> {
    ops: BTreeSet<(Timestamp, Operation<Val>)>,
}

impl<Val: Ord> List<Val> {
    pub fn new_from_root_id(root_id: Timestamp) -> Self {
        Self {
            ops: BTreeSet::from([(root_id, Operation::MakeList)]),
        }
    }
}
