use crate::timestamp::Timestamp;

#[derive(Debug)]
pub struct Order {
    root_id: Timestamp,
    values: Vec<Timestamp>,
}

impl Order {
    pub fn new(root_id: Timestamp) -> Self {
        Self {
            root_id,
            values: Vec::new(),
        }
    }

    pub fn insert_after(&mut self, op_id: Timestamp, after: Timestamp) {
        if after == self.root_id {
            self.values.insert(0, op_id);
        } else {
            for (i, value) in self.values.iter().enumerate() {
                if value == &after {
                    self.values.insert(i + 1, op_id);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn new_is_empty() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let order = Order::new(root_id);

        assert_eq!(order.root_id, root_id);
        assert!(order.values.is_empty());
    }

    #[test]
    fn insert_after_root_id() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let mut order = Order::new(root_id);

        let op_id = Timestamp::new(1, Uuid::nil());
        order.insert_after(op_id, root_id);

        assert_eq!(order.values, vec![op_id]);
    }

    #[test]
    fn insert_after_other_item() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let mut order = Order::new(root_id);

        let op_a = Timestamp::new(1, Uuid::nil());
        order.insert_after(op_a, root_id);

        let op_b = Timestamp::new(2, Uuid::nil());
        order.insert_after(op_b, op_a);

        assert_eq!(order.values, vec![op_a, op_b]);
    }

    #[test]
    fn insert_after_not_present() {
        let root_id = Timestamp::new(0, Uuid::nil());
        let mut order = Order::new(root_id);

        order.insert_after(
            Timestamp::new(1, Uuid::nil()),
            // This ID doesn't exist in the ordering!
            Timestamp::new(2, Uuid::nil()),
        );

        assert_eq!(order.values, vec![]);
    }
}
