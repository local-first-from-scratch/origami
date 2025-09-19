use serde::{Deserialize, Serialize};

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize, Clone, Copy)]
pub struct Hlc(u64);

const TIMESTAMP_BITS: u32 = 32;
const TIMESTAMP_MASK: u64 = !((u32::MAX as u64) << TIMESTAMP_BITS);

const COUNTER_BITS: u32 = 16;
const COUNTER_MASK: u64 = !((u16::MAX as u64) << COUNTER_BITS);

const NODE_BITS: u32 = 16;
const NODE_MASK: u64 = !(u16::MAX as u64);

impl Hlc {
    pub fn new_at(timestamp: u32, counter: u16, node: u16) -> Self {
        let mut out: u64 = 0;

        out |= timestamp as u64;

        out <<= COUNTER_BITS;
        out |= counter as u64;

        out <<= NODE_BITS;
        out |= node as u64;

        Self(out)
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub fn next(&self, new_timestamp: u32) -> Self {
        // If time is in the past, increment the counter. We don't care if we
        // overflow the bit field used for the counter; we'll always be
        // incrementing in the same way so it's fine. The timestamp is only
        // approximately accurate anyway!
        if new_timestamp <= self.timestamp() {
            Hlc(self.0 + (1 << COUNTER_BITS))
        } else {
            Hlc(self.0 & TIMESTAMP_MASK & COUNTER_MASK | ((new_timestamp as u64) << TIMESTAMP_BITS))
        }
    }

    pub fn set_node(&self, new_node: u16) -> Self {
        Self(self.0 & NODE_MASK | new_node as u64)
    }

    #[inline]
    pub fn timestamp(&self) -> u32 {
        (self.0 >> TIMESTAMP_BITS) as u32
    }

    #[inline]
    pub fn counter(&self) -> u16 {
        (self.0 >> COUNTER_BITS) as u16
    }

    #[inline]
    pub fn node(&self) -> u16 {
        self.0 as u16
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn create() {
        assert_eq!(
            Hlc::new_at(1, 1, 1),
            Hlc(0b100000000000000010000000000000001)
        )
    }

    #[test]
    fn get_timestamp() {
        let hlc = Hlc::new_at(123, 0, 0);

        assert_eq!(hlc.timestamp(), 123);
        assert_eq!(hlc.counter(), 0);
    }

    #[test]
    fn get_counter() {
        let hlc = Hlc::new_at(0, 123, 0);

        assert_eq!(hlc.counter(), 123)
    }

    #[test]
    fn get_node() {
        let hlc = Hlc::new_at(0, 0, 123);

        assert_eq!(hlc.node(), 123)
    }

    #[test]
    fn next_in_past_increments_counter() {
        let hlc = Hlc::new_at(1, 0, 3).next(0);

        assert_eq!(hlc.timestamp(), 1);
        assert_eq!(hlc.counter(), 1);
        assert_eq!(hlc.node(), 3);
    }

    #[test]
    fn next_in_past_with_full_counter_rolls_over() {
        let hlc = Hlc::new_at(0, u16::MAX, 3).next(0);

        assert_eq!(hlc.timestamp(), 1);
        assert_eq!(hlc.counter(), 0);
        assert_eq!(hlc.node(), 3);
    }

    #[test]
    fn next_in_future_sets_timestamp_to_given_value_and_resets_counter() {
        let hlc = Hlc::new_at(0, 8, 3).next(1);

        assert_eq!(hlc.timestamp(), 1);
        assert_eq!(hlc.counter(), 0);
        assert_eq!(hlc.node(), 3);
    }

    #[test]
    fn set_node() {
        let hlc = Hlc::new_at(0, 0, 1).set_node(2);

        assert_eq!(hlc.timestamp(), 0);
        assert_eq!(hlc.counter(), 0);
        assert_eq!(hlc.node(), 2);
    }

    #[test]
    fn ord_timestamp_first() {
        let a = Hlc::new_at(0, 1, 1);
        let b = Hlc::new_at(1, 0, 0);

        assert!(a < b, "{a:?} was not less than {b:?}")
    }

    #[test]
    fn ord_counter_second() {
        let a = Hlc::new_at(0, 0, 1);
        let b = Hlc::new_at(0, 1, 0);

        assert!(a < b, "{a:?} was not less than {b:?}")
    }

    #[test]
    fn ord_node_third() {
        let a = Hlc::new_at(0, 0, 0);
        let b = Hlc::new_at(0, 0, 1);

        assert!(a < b, "{a:?} was not less than {b:?}")
    }
}
