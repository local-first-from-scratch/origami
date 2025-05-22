mod assign;
mod operation;
mod order;

use crate::timestamp::Timestamp;
use assign::Assign;
pub use operation::AssignKey;
use operation::Operation;
use order::Order;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug)]
pub struct Document<Val: Ord + Clone> {
    operations: BTreeSet<(Timestamp, Operation<Val>)>,

    ordering: Order,
    assignment: BTreeMap<Timestamp, Assign>,
    values: BTreeMap<Timestamp, Val>,

    highest_counter: u64,
}

impl<Val: Ord + Clone> Document<Val> {
    pub fn new() -> Self {
        Self {
            operations: BTreeSet::new(),

            ordering: Order::new(),
            assignment: BTreeMap::new(),
            values: BTreeMap::new(),

            highest_counter: 0,
        }
    }

    fn next_timestamp_counter(&mut self) -> u64 {
        // note for later: we'll have to do a `max` between this and any
        // incoming operations. The increment will be the same, though.
        self.highest_counter += 1;

        self.highest_counter
    }

    pub fn root(&mut self) -> Option<&Timestamp> {
        for (id, op) in &self.operations {
            if matches!(op, Operation::MakeMap | Operation::MakeList) {
                return Some(id);
            }
        }

        None
    }

    fn apply(&mut self, id: Timestamp, operation: &Operation<Val>) {
        match operation {
            Operation::MakeMap | Operation::MakeList => {
                debug_assert!(
                    !self.assignment.contains_key(&id),
                    "document already contains {id}"
                );

                self.assignment.insert(id, Assign::default());
            }

            Operation::MakeVal { val } => {
                debug_assert!(
                    !self.values.contains_key(&id),
                    "document already contains val {id}"
                );

                self.values.insert(id, val.clone());
            }

            Operation::Assign {
                obj,
                key,
                val,
                prev,
            } => {
                self.assignment
                    .entry(*obj)
                    .or_default()
                    .assign(id, key.clone(), *val, prev);
            }

            Operation::InsertAfter { prev } => {
                self.ordering.insert_after(id, *prev);
            }

            Operation::Remove { obj, key, prev } => {
                self.assignment
                    .entry(*obj)
                    .and_modify(|a| a.remove(key, prev));
            }
        };
    }

    pub fn make_map(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeMap;

        self.apply(id, &op);
        self.operations.insert((id, op));

        id
    }

    pub fn make_list(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeList;

        self.apply(id, &op);
        self.operations.insert((id, op));

        id
    }

    pub fn make_val(&mut self, val: Val, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeVal { val };

        self.apply(id, &op);
        self.operations.insert((id, op));

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
        let op = Operation::Assign {
            obj,
            key: key.clone(),
            val,
            prev: prev.clone(),
        };

        self.apply(id, &op);
        self.operations.insert((id, op));

        id
    }

    pub fn insert_after(&mut self, prev: Timestamp, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::InsertAfter { prev };

        self.apply(id, &op);
        self.operations.insert((id, op));

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
        let op = Operation::Remove { obj, key, prev };

        self.apply(id, &op);
        self.operations.insert((id, op));

        id
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn make_map_gives_timestamp_for_map() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        let map_id = doc.make_map(node_id);

        // The timestamp should now exist in the document
        assert!(doc.assignment.contains_key(&map_id));
    }

    #[test]
    fn make_list_gives_timestamp_for_list() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        let list_id = doc.make_list(node_id);

        // The timestamp should now exist in the document
        assert!(doc.assignment.contains_key(&list_id));
    }

    #[test]
    fn make_val_gives_timestamp_for_val() {
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();
        let value = 0;

        let val_id = doc.make_val(value, node_id);

        // The timestamp should now exist in the document
        assert_eq!(doc.values.get(&val_id), Some(&value));
    }

    #[test]
    fn assign_to_non_existent_object_stores_anyway() {
        // TODO: I'm not sure if this is the correct behavior. Should we store
        // random keys where we haven't seen an equivalent `MakeMap`? I could
        // see arguments both directions. On one hand, we want to be resistant
        // to buggy behavior. On the other hand, once an operation has been
        // accepted we need to acknowledge and deal with it.
        let mut doc = Document::<i32>::new();
        let node_id = Uuid::new_v4();

        // Create a timestamp that doesn't exist in the document
        let non_existent_id = Timestamp::new(999, node_id);

        // Create a value to assign
        let val_id = doc.make_val(0, node_id);

        // Try to assign the value to a non-existent object
        doc.assign(
            non_existent_id,
            AssignKey::MapKey("key".to_string()),
            val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that the assignment entry was created for the non-existent object
        assert!(doc.assignment.contains_key(&non_existent_id));
    }

    #[test]
    fn assigning_then_removing_results_in_removal() {
        let mut doc = Document::<i32>::new();
        let node = Uuid::nil();

        let map_id = doc.make_map(node);
        let val = doc.make_val(1, node);
        let key = AssignKey::MapKey("test".into());

        let assign_id = doc.assign(map_id, key.clone(), val, BTreeSet::new(), node);
        doc.remove(map_id, key.clone(), BTreeSet::from([assign_id]), node);

        assert!(
            doc.assignment
                .get(&map_id)
                .and_then(|a| a.get(&key))
                .is_none(),
            "{key:?} was unexpectedly still present for map {map_id} in doc {doc:#?}",
        );
    }
}
