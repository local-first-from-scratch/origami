use std::fmt::Display;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

impl Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.counter, self.node)
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

    #[test]
    fn display_includes_counter_and_timestamp() {
        let ts = Timestamp::new(0, Uuid::nil());

        assert_eq!(ts.to_string(), "0@00000000-0000-0000-0000-000000000000")
    }
}
