use super::list::List;
use super::map::Map;
use crate::timestamp::Timestamp;

#[derive(Debug)]
pub enum Object<Val: Ord> {
    Map(Map<Val>),
    List(List<Val>),
    Val(Timestamp, Val),
}
