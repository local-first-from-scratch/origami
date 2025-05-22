use crate::timestamp::Timestamp;
use std::collections::BTreeSet;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Clone)]
#[repr(C)]
pub enum Operation<Val: Clone> {
    MakeMap,
    MakeList,
    MakeVal {
        val: Val,
    },
    InsertAfter {
        prev: Timestamp,
    },
    Assign {
        obj: Timestamp,
        key: AssignKey,
        val: Timestamp,
        prev: BTreeSet<Timestamp>,
    },
    Remove {
        obj: Timestamp,
        key: AssignKey,
        prev: BTreeSet<Timestamp>,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[repr(C)]
pub enum AssignKey {
    // TODO: numeric keys as well?
    MapKey(String),
    InsertAfter(Timestamp),
}
