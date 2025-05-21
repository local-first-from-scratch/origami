use crate::timestamp::Timestamp;
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct Order {
    ordering: BTreeMap<Timestamp, Timestamp>,
}

impl Order {
    pub fn new() -> Self {
        Self {
            ordering: BTreeMap::new(),
        }
    }

    pub fn insert_after(&mut self, op_id: Timestamp, after: Timestamp) {
        if let Some(previous) = self.ordering.insert(after, op_id) {
            self.ordering.insert(op_id, previous);
        }
    }

    pub fn iter<'o>(&'o self, start: &'o Timestamp) -> OrderIterator<'o> {
        OrderIterator::new(self, start)
    }
}

#[derive(Debug)]
pub struct OrderIterator<'o> {
    order: &'o Order,
    current: Option<&'o Timestamp>,
}

impl<'o> OrderIterator<'o> {
    pub fn new(order: &'o Order, start: &'o Timestamp) -> Self {
        Self {
            order,
            // We don't want to iterate over the start of the list; it's the
            // Timestamp of a `MakeList` operation, not an actual list item.
            current: order.ordering.get(start),
        }
    }
}

impl<'o> Iterator for OrderIterator<'o> {
    type Item = &'o Timestamp;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.and_then(|c| self.order.ordering.get(c)) {
            Some(next) => self.current.replace(next),
            None => self.current.take(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::{prop_assume, proptest};

    proptest! {
        // In non-byzantine circumstances, we should move timestamps around so
        // that we have a single chains with neither branching nor joining. "No
        // branching" is guaranteed by our data structure (we cannot have an
        // entry with two values), but joining is not ruled out because you can
        // have multiple keys pointing at the same value. Well, let's test to
        // rule that out!
        #[test]
        fn insert_after_never_has_duplicate_values(values: Vec<Timestamp>) {
            let mut order = Order::new();
            values.windows(2).for_each(|window| order.insert_after(window[1], window[0]));

            let mut count: BTreeMap<Timestamp, usize> = BTreeMap::new();
            for v in order.ordering.values() {
                *count.entry(*v).or_default() += 1
            }

            for (k, v) in count {
                assert_eq!(v, 1, "{k} had multiple incoming pointers. Full data structure: {order:#?}")
            }
        }
    }

    proptest! {
        #[test]
        fn iteration_retains_ordering(values: Vec<Timestamp>) {
            prop_assume!(values.len() > 0);

            let mut order = Order::new();
            values.windows(2).for_each(|window| order.insert_after(window[1], window[0]));

            assert_eq!(values[1..], order.iter(&values[0]).copied().collect::<Vec<Timestamp>>())
        }
    }
}
