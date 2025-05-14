use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
pub struct Timestamp {
    pub counter: u64,
    #[cfg_attr(test, proptest(strategy = "crate::test_helpers::uuid()"))]
    pub node: Uuid,
}

impl Timestamp {
    pub fn new(counter: u64, node: Uuid) -> Self {
        Self { counter, node }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::proptest;

    proptest! {
        #[test]
        fn sort_by_counter_then_node(a: Timestamp, b: Timestamp) {
            if a.counter != b.counter {
                assert_eq!(a.counter.cmp(&b.counter), a.cmp(&b));
            } else {
                assert_eq!(a.node.cmp(&b.node), a.cmp(&b));
            }
        }
    }
}
