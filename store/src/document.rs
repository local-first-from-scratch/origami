mod assign;
mod operation;
mod order;
mod value;

use crate::timestamp::Timestamp;
use assign::Assign;
pub use operation::AssignKey;
use operation::Operation;
use order::Order;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
pub use value::{NULL, Value, ValueError};

#[derive(Debug, Default)]
pub struct Document {
    operations: Vec<(Timestamp, Operation)>,

    maps: BTreeMap<Timestamp, Assign<String>>,

    list_items: BTreeMap<Timestamp, Assign<Timestamp>>,
    list_ordering: Order,

    values: BTreeMap<Timestamp, Value>,

    highest_counter: u64,
}

impl Document {
    fn next_timestamp_counter(&mut self) -> u64 {
        // note for later: we'll have to do a `max` between this and any
        // incoming operations. The increment will be the same, though.
        self.highest_counter += 1;

        self.highest_counter
    }

    pub fn root(&self) -> Option<&Timestamp> {
        for (id, op) in &self.operations {
            if matches!(op, Operation::MakeMap | Operation::MakeList) {
                return Some(id);
            }
        }

        None
    }

    fn apply(&mut self, id: Timestamp, operation: &Operation) {
        match operation {
            Operation::MakeMap => {
                debug_assert!(
                    !self.maps.contains_key(&id),
                    "document.maps already contains {id}"
                );

                self.maps.insert(id, Assign::new());
            }

            Operation::MakeList => {
                debug_assert!(
                    !self.maps.contains_key(&id),
                    "document.lists already contains {id}"
                );

                self.list_items.insert(id, Assign::new());
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
                match key {
                    AssignKey::MapKey(key) => {
                        self.maps
                            .entry(*obj)
                            .or_default()
                            .assign(id, key.clone(), *val, prev)
                    }
                    AssignKey::InsertAfter(timestamp) => self
                        .list_items
                        .entry(*obj)
                        .or_insert_with(Assign::new)
                        .assign(id, *timestamp, *val, prev),
                };
            }

            Operation::InsertAfter { prev } => {
                self.list_ordering.insert_after(id, *prev);
            }

            Operation::Remove { obj, key, prev } => match key {
                AssignKey::MapKey(key) => {
                    self.maps.entry(*obj).and_modify(|a| a.remove(key, prev));
                }
                AssignKey::InsertAfter(timestamp) => {
                    self.list_items
                        .entry(*obj)
                        .and_modify(|a| a.remove(timestamp, prev));
                }
            },
        };
    }

    pub fn make_map(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeMap;

        self.apply(id, &op);
        self.operations.push((id, op));

        id
    }

    pub fn make_list(&mut self, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeList;

        self.apply(id, &op);
        self.operations.push((id, op));

        id
    }

    pub fn make_val(&mut self, val: Value, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeVal { val };

        self.apply(id, &op);
        self.operations.push((id, op));

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
        self.operations.push((id, op));

        id
    }

    pub fn insert_after(&mut self, prev: Timestamp, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::InsertAfter { prev };

        self.apply(id, &op);
        self.operations.push((id, op));

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
        self.operations.push((id, op));

        id
    }

    pub fn as_value(&self) -> Value {
        match self.root() {
            None => NULL,
            Some(root) => self.get(root),
        }
    }

    fn get(&self, id: &Timestamp) -> Value {
        if self.maps.contains_key(id) {
            self.get_map(id)
        } else if self.list_items.contains_key(id) {
            self.get_list(id)
        } else if let Some(val) = self.values.get(id) {
            val.clone()
        } else {
            NULL
        }
    }

    fn get_map(&self, id: &Timestamp) -> Value {
        let mut map = BTreeMap::new();

        if let Some(assign) = self.maps.get(id) {
            for (k, v) in assign.iter_map() {
                if v.len() == 1 {
                    map.insert(k.to_string(), self.get(v[0]));
                } else {
                    todo!("multiple-valued key in map")
                }
            }
        }

        Value::Map(map)
    }

    fn get_list(&self, id: &Timestamp) -> Value {
        let mut list = Vec::new();

        if let Some(assign) = self.list_items.get(id) {
            for item_id in self.list_ordering.iter(id) {
                if let Some(values) = assign.get(item_id) {
                    if values.len() == 1 {
                        list.push(self.get(values.first_key_value().unwrap().1))
                    } else {
                        todo!("multiple-valued key in list")
                    }
                }
            }
        }

        Value::List(list)
    }

    pub fn current_assigns(&self, id: &Timestamp, assign_key: &AssignKey) -> BTreeSet<Timestamp> {
        match assign_key {
            AssignKey::MapKey(key) => match self.maps.get(id) {
                Some(assign) => assign
                    .get(key)
                    .map(|ks| ks.keys().copied().collect())
                    .unwrap_or_else(BTreeSet::new),
                // Returning an empty set here because this method is used to
                // get the `prev` value for assign calls.
                None => BTreeSet::new(),
            },
            AssignKey::InsertAfter(_insert_after) => todo!(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn make_map_gives_timestamp_for_map() {
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();

        let map_id = doc.make_map(node_id);

        // The timestamp should now exist in the document
        assert!(doc.maps.contains_key(&map_id));
    }

    #[test]
    fn make_list_gives_timestamp_for_list() {
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();

        let list_id = doc.make_list(node_id);

        // The timestamp should now exist in the document
        assert!(doc.list_items.contains_key(&list_id));
    }

    #[test]
    fn make_val_gives_timestamp_for_val() {
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();
        let value = Value::from(0);

        let val_id = doc.make_val(value.clone(), node_id);

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
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();

        // Create a timestamp that doesn't exist in the document
        let non_existent_id = Timestamp::new(999, node_id);

        // Create a value to assign
        let val_id = doc.make_val(0.into(), node_id);

        // Try to assign the value to a non-existent object
        doc.assign(
            non_existent_id,
            AssignKey::MapKey("key".to_string()),
            val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that the assignment entry was created for the non-existent object
        assert!(doc.maps.contains_key(&non_existent_id));
    }

    #[test]
    fn assigning_then_removing_results_in_removal() {
        let mut doc = Document::default();
        let node = Uuid::nil();

        let map_id = doc.make_map(node);
        let val = doc.make_val(1.into(), node);

        let key = "test".to_string();
        let assign_key = AssignKey::MapKey(key.clone());

        let assign_id = doc.assign(map_id, assign_key.clone(), val, BTreeSet::new(), node);
        doc.remove(map_id, assign_key, BTreeSet::from([assign_id]), node);

        assert!(
            doc.maps.get(&map_id).and_then(|a| a.get(&key)).is_none(),
            "{key:?} was unexpectedly still present for map {map_id} in doc {doc:#?}",
        );
    }
}
