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

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::BTreeMap;
    use uuid::Uuid;

    #[test]
    fn assign_assigns_current_value() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let key = AssignKey::MapKey("test".into());

        let op_id = Timestamp::new(2, Uuid::nil());
        let value_id = Timestamp::new(1, Uuid::nil());

        let mut map: Map<()> = Map::new_from_root_id(root_id);

        map.assign(op_id, root_id, key.clone(), value_id, BTreeSet::new());

        assert_eq!(
            map.current_values.get(&key),
            Some(&BTreeMap::from([(op_id, value_id)]))
        );
    }

    #[test]
    fn inserts_op() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let key = AssignKey::MapKey("test".into());

        let op_id = Timestamp::new(2, Uuid::nil());
        let value_id = Timestamp::new(1, Uuid::nil());

        let mut map: Map<()> = Map::new_from_root_id(root_id);

        map.assign(op_id, root_id, key.clone(), value_id, BTreeSet::new());

        assert_eq!(
            map.ops.last(),
            Some(&(
                op_id,
                Operation::Assign {
                    obj: root_id,
                    key,
                    val: value_id,
                    prev: BTreeSet::new()
                }
            ))
        );
    }
}
