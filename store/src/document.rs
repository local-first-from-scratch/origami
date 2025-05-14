use crate::operation::Operation;
use crate::timestamp::Timestamp;
use facet::Facet;
use std::collections::BTreeSet;

#[derive(Facet)]
pub struct Document<Val: Ord> {
    operations: BTreeSet<(Timestamp, Operation<Val>)>,
}
