mod assign;
mod operation;
mod order;
mod value;

use crate::timestamp::Timestamp;
use assign::Assign;
use json_patch::{AddOperation, PatchOperation, jsonptr::PointerBuf};
pub use operation::AssignKey;
use operation::Operation;
use order::Order;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;
pub use value::{Value, ValueError};

#[derive(Debug, Default)]
pub struct Document {
    operations: Vec<(Timestamp, Operation)>,

    maps: BTreeMap<Timestamp, Assign<String>>,

    list_items: BTreeMap<Timestamp, Assign<Timestamp>>,
    list_ordering: Order,

    values: BTreeMap<Timestamp, (Value, String)>,

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
            if matches!(op, Operation::MakeMap { .. } | Operation::MakeList { .. }) {
                return Some(id);
            }
        }

        None
    }

    fn apply(&mut self, id: Timestamp, operation: &Operation) {
        match operation {
            Operation::MakeMap { schema } => {
                debug_assert!(
                    !self.maps.contains_key(&id),
                    "document.maps already contains {id}"
                );

                self.maps.insert(id, Assign::new(schema.clone()));
            }

            Operation::MakeList { schema } => {
                debug_assert!(
                    !self.maps.contains_key(&id),
                    "document.lists already contains {id}"
                );

                self.list_items.insert(id, Assign::new(schema.clone()));
            }

            Operation::MakeVal { val, schema } => {
                debug_assert!(
                    !self.values.contains_key(&id),
                    "document already contains val {id}"
                );

                // TODO: could we use references here?
                self.values.insert(id, (val.clone(), schema.clone()));
            }

            Operation::Assign {
                obj,
                key,
                val,
                prev,
            } => {
                match key {
                    AssignKey::MapKey(key) => {
                        if let Some(map) = self.maps.get_mut(obj) {
                            map.assign(id, key.clone(), *val, prev)
                        }
                    }
                    AssignKey::InsertAfter(timestamp) => {
                        if let Some(list) = self.list_items.get_mut(obj) {
                            list.assign(id, *timestamp, *val, prev)
                        }
                    }
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

    pub fn make_map(&mut self, schema: String, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeMap { schema };

        self.apply(id, &op);
        self.operations.push((id, op));

        id
    }

    pub fn make_list(&mut self, schema: String, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeList { schema };

        self.apply(id, &op);
        self.operations.push((id, op));

        id
    }

    pub fn make_val(&mut self, val: Value, schema: String, node: Uuid) -> Timestamp {
        let id = Timestamp::new(self.next_timestamp_counter(), node);
        let op = Operation::MakeVal {
            val,
            schema: schema.clone(),
        };

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

    pub fn as_patch(&self) -> Vec<PatchOperation> {
        let mut ops = Vec::new();
        let here = PointerBuf::new();

        if let Some(root) = self.root() {
            self.push_patch(root, here, &mut ops)
        }

        ops
    }

    fn push_patch(&self, id: &Timestamp, here: PointerBuf, ops: &mut Vec<PatchOperation>) {
        if self.maps.contains_key(id) {
            self.push_map_patches(id, here, ops)
        } else if self.list_items.contains_key(id) {
            self.push_list_patches(id, here, ops)
        } else if let Some((val, _schema)) = self.values.get(id) {
            ops.push(PatchOperation::Add(AddOperation {
                path: here,
                value: val.into(),
            }))
        } else {
            // TODO: a better error if we don't have all the values we expected.
            // This shouldn't happen (it can currently happen if you remove an
            // item from a list, because we remove the item from the assign but
            // not the linked list used for ordering.)
            todo!("we don't have all the values we expected")
        }
    }

    /// Push JSON patches to the `ops`, assuming that the ID has already been
    /// validated as a map. If that assumption does not hold, this is a no-op.
    fn push_map_patches(&self, id: &Timestamp, here: PointerBuf, ops: &mut Vec<PatchOperation>) {
        if let Some(assign) = self.maps.get(id) {
            if !here.is_root() {
                ops.push(PatchOperation::Add(AddOperation {
                    path: here.clone(),
                    value: json!({}),
                }));
            }

            for (k, v) in assign.iter_map() {
                if v.len() == 1 {
                    self.push_patch(v[0], here.with_trailing_token(k), ops);
                } else {
                    todo!("multiple-valued key in map")
                }
            }
        }
    }

    /// Push JSON patches to the `ops`, assuming that the ID has already been validated as a list.
    fn push_list_patches(&self, id: &Timestamp, here: PointerBuf, ops: &mut Vec<PatchOperation>) {
        if let Some(assign) = self.list_items.get(id) {
            if !here.is_root() {
                ops.push(PatchOperation::Add(AddOperation {
                    path: here.clone(),
                    value: json!([]),
                }));
            }

            for (index, item_id) in self.list_ordering.iter(id).enumerate() {
                if let Some(values) = assign.get(item_id) {
                    if values.len() == 1 {
                        self.push_patch(
                            values.first_key_value().unwrap().1,
                            here.with_trailing_token(index),
                            ops,
                        )
                    } else {
                        todo!("multiple-valued key in list")
                    }
                }
            }
        }
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

        let map_id = doc.make_map("test".into(), node_id);

        // The timestamp should now exist in the document
        assert!(doc.maps.contains_key(&map_id));
    }

    #[test]
    fn make_list_gives_timestamp_for_list() {
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();

        let list_id = doc.make_list("test".into(), node_id);

        // The timestamp should now exist in the document
        assert!(doc.list_items.contains_key(&list_id));
    }

    #[test]
    fn make_val_gives_timestamp_for_val() {
        let mut doc = Document::default();
        let node_id = Uuid::new_v4();
        let value = Value::from(0);

        let val_id = doc.make_val(value.clone(), "test".into(), node_id);

        // The timestamp should now exist in the document
        assert_eq!(doc.values.get(&val_id), Some(&(value, "test".into())));
    }

    #[test]
    fn assign_to_non_existent_object_skips_application() {
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
        let val_id = doc.make_val(0.into(), "test".into(), node_id);

        // Try to assign the value to a non-existent object
        doc.assign(
            non_existent_id,
            AssignKey::MapKey("key".to_string()),
            val_id,
            BTreeSet::new(),
            node_id,
        );

        // Check that the assignment entry was created for the non-existent object
        assert!(!doc.maps.contains_key(&non_existent_id));
    }

    #[test]
    fn assigning_then_removing_results_in_removal() {
        let mut doc = Document::default();
        let node = Uuid::nil();

        let map_id = doc.make_map("test".into(), node);
        let val = doc.make_val(1.into(), "test".into(), node);

        let key = "test".to_string();
        let assign_key = AssignKey::MapKey(key.clone());

        let assign_id = doc.assign(map_id, assign_key.clone(), val, BTreeSet::new(), node);
        doc.remove(map_id, assign_key, BTreeSet::from([assign_id]), node);

        assert!(
            doc.maps.get(&map_id).and_then(|a| a.get(&key)).is_none(),
            "{key:?} was unexpectedly still present for map {map_id} in doc {doc:#?}",
        );
    }

    mod as_patch {
        use super::*;
        use pretty_assertions::assert_eq;
        use serde_json::from_value;

        macro_rules! patch {
            ($patch:tt) => {
                from_value::<json_patch::Patch>(json!($patch)).unwrap().0
            };
        }

        #[test]
        fn object_root() {
            let mut doc = Document::default();
            doc.make_map("test".into(), Uuid::nil());

            assert_eq!(doc.as_patch(), patch!([]));
        }

        #[test]
        fn object_assign() {
            let mut doc = Document::default();
            let root_id = doc.make_map("test".into(), Uuid::nil());
            let val_id = doc.make_val("world".into(), "test".into(), Uuid::nil());
            doc.assign(
                root_id,
                AssignKey::MapKey("hello".to_string()),
                val_id,
                BTreeSet::new(),
                Uuid::nil(),
            );

            assert_eq!(
                doc.as_patch(),
                patch!([
                    { "op": "add", "path": "/hello", "value": "world" },
                ])
            );
        }

        #[test]
        fn list_assign() {
            let mut doc = Document::default();
            let root_id = doc.make_list("test".into(), Uuid::nil());

            let val_a = doc.make_val("hello".into(), "test".into(), Uuid::nil());
            let insert_a = doc.insert_after(root_id, Uuid::nil());
            doc.assign(
                root_id,
                AssignKey::InsertAfter(insert_a),
                val_a,
                BTreeSet::new(),
                Uuid::nil(),
            );

            let val_b = doc.make_val("howdy".into(), "test".into(), Uuid::nil());
            let insert_b = doc.insert_after(insert_a, Uuid::nil());
            doc.assign(
                root_id,
                AssignKey::InsertAfter(insert_b),
                val_b,
                BTreeSet::new(),
                Uuid::nil(),
            );

            assert_eq!(
                doc.as_patch(),
                patch!([
                    { "op": "add", "path": "/0", "value": "hello" },
                    { "op": "add", "path": "/1", "value": "howdy" },
                ])
            );
        }

        #[test]
        fn deep_assign_map() {
            let mut doc = Document::default();
            let root_id = doc.make_map("test".into(), Uuid::nil());

            let greetings_id = doc.make_map("test".into(), Uuid::nil());
            doc.assign(
                root_id,
                AssignKey::MapKey("greetings".into()),
                greetings_id,
                BTreeSet::new(),
                Uuid::nil(),
            );

            let world_id = doc.make_val("world".into(), "test".into(), Uuid::nil());
            doc.assign(
                greetings_id,
                AssignKey::MapKey("hello".to_string()),
                world_id,
                BTreeSet::new(),
                Uuid::nil(),
            );

            assert_eq!(
                doc.as_patch(),
                patch!([
                    { "op": "add", "path": "/greetings", "value": {} },
                    { "op": "add", "path": "/greetings/hello", "value": "world" },
                ])
            );
        }

        #[test]
        fn deep_assign_list() {
            let mut doc = Document::default();
            let root_id = doc.make_map("test".into(), Uuid::nil());

            let greetings_id = doc.make_list("test".into(), Uuid::nil());
            doc.assign(
                root_id,
                AssignKey::MapKey("greetings".into()),
                greetings_id,
                BTreeSet::new(),
                Uuid::nil(),
            );

            let insert_id = doc.insert_after(greetings_id, Uuid::nil());
            let world_id = doc.make_val("world".into(), "test".into(), Uuid::nil());
            doc.assign(
                greetings_id,
                AssignKey::InsertAfter(insert_id),
                world_id,
                BTreeSet::new(),
                Uuid::nil(),
            );

            assert_eq!(
                doc.as_patch(),
                patch!([
                    { "op": "add", "path": "/greetings", "value": [] },
                    { "op": "add", "path": "/greetings/0", "value": "world" },
                ])
            );
        }
    }
}
