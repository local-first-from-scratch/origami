use super::AssignKey;
use crate::timestamp::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug)]
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
}
