use super::list::List;
use super::map::Map;

pub enum Object<Val: Ord> {
    Map(Map),
    List(List),
    Val(Val),
}
