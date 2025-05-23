use super::AssignKey;
use crate::timestamp::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct Assign {
    values: BTreeMap<AssignKey, BTreeMap<Timestamp, Timestamp>>,
}

impl Assign {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    pub fn assign(
        &mut self,
        id: Timestamp,
        key: AssignKey,
        val: Timestamp,
        prev: &BTreeSet<Timestamp>,
    ) {
        let entry = self.values.entry(key).or_default();
        for prev_id in prev {
            entry.remove(&prev_id);
        }
        entry.insert(id, val);
    }

    pub fn remove(&mut self, key: &AssignKey, prev: &BTreeSet<Timestamp>) {
        if let Some(values) = self.values.get_mut(key) {
            for prev_key in prev {
                values.remove(prev_key);
            }

            if values.is_empty() {
                self.values.remove(key);
            }
        }
    }

    pub fn get(&self, key: &AssignKey) -> Option<&BTreeMap<Timestamp, Timestamp>> {
        self.values.get(key)
    }

    pub fn iter_map(&self) -> impl Iterator<Item = (&str, Vec<&Timestamp>)> {
        self.values.iter().filter_map(|(k, v)| match k {
            AssignKey::MapKey(key) => Some((key.as_str(), v.values().collect())),
            AssignKey::InsertAfter(..) => None,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn assigning_to_empty_struct_retains_value() {
        let mut assign = Assign::new();

        let key = AssignKey::MapKey("a".into());
        let operation_id = Timestamp::new(0, Uuid::nil());
        let value = Timestamp::new(1, Uuid::nil());

        assign.assign(operation_id, key.clone(), value, &BTreeSet::new());

        assert_eq!(
            assign.values.get(&key),
            Some(&BTreeMap::from([(operation_id, value)]))
        )
    }

    #[test]
    fn parallel_assignments_keep_both_values() {
        let mut assign = Assign::new();

        let key = AssignKey::MapKey("a".into());

        let operation_id_a = Timestamp::new(0, Uuid::nil());
        let value_a = Timestamp::new(1, Uuid::nil());

        let operation_id_b = Timestamp::new(2, Uuid::nil());
        let value_b = Timestamp::new(3, Uuid::nil());

        // These assignments are parallel because neither has "prev"!
        assign.assign(operation_id_a, key.clone(), value_a, &BTreeSet::new());
        assign.assign(operation_id_b, key.clone(), value_b, &BTreeSet::new());

        assert_eq!(
            assign.values.get(&key),
            Some(&BTreeMap::from([
                (operation_id_a, value_a),
                (operation_id_b, value_b)
            ]))
        )
    }

    #[test]
    fn prev_removes_existing_assignment() {
        let mut assign = Assign::new();

        let key = AssignKey::MapKey("a".into());

        let operation_id_a = Timestamp::new(0, Uuid::nil());
        let value_a = Timestamp::new(1, Uuid::nil());

        let operation_id_b = Timestamp::new(2, Uuid::nil());
        let value_b = Timestamp::new(3, Uuid::nil());

        // These assignments are parallel because neither has "prev"!
        assign.assign(operation_id_a, key.clone(), value_a, &BTreeSet::new());
        assign.assign(
            operation_id_b,
            key.clone(),
            value_b,
            &BTreeSet::from([operation_id_a]),
        );

        assert_eq!(
            assign.values.get(&key),
            Some(&BTreeMap::from([(operation_id_b, value_b)]))
        )
    }

    #[test]
    fn remove_removes_only_values_indicated_by_prev() {
        let mut assign = Assign::new();

        let key = AssignKey::MapKey("a".into());

        let operation_id_a = Timestamp::new(0, Uuid::nil());
        let value_a = Timestamp::new(1, Uuid::nil());

        let operation_id_b = Timestamp::new(2, Uuid::nil());
        let value_b = Timestamp::new(3, Uuid::nil());

        // Add two values
        assign.assign(operation_id_a, key.clone(), value_a, &BTreeSet::new());
        assign.assign(operation_id_b, key.clone(), value_b, &BTreeSet::new());

        // Remove only the first value
        assign.remove(&key, &BTreeSet::from([operation_id_a]));

        // Verify that only the second value remains
        assert_eq!(
            assign.values.get(&key),
            Some(&BTreeMap::from([(operation_id_b, value_b)]))
        );
    }

    #[test]
    fn remove_removes_the_entire_key_if_all_values_are_removed() {
        let mut assign = Assign::new();

        let key = AssignKey::MapKey("a".into());

        let operation_id = Timestamp::new(0, Uuid::nil());
        let value = Timestamp::new(1, Uuid::nil());

        // Add a value
        assign.assign(operation_id, key.clone(), value, &BTreeSet::new());

        // Remove the value
        assign.remove(&key, &BTreeSet::from([operation_id]));

        // Verify that the key is no longer in the map
        assert_eq!(assign.values.get(&key), None);
    }
}
