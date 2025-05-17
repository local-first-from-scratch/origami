use crate::timestamp::Timestamp;

#[derive(Debug)]
pub struct Map {
    // TODO: not happy with this at the moment. It seems like it's going to make
    // it harder to retrieve all the operations if we have to special-case
    // things like this. Might still be marginally better than storing all the
    // operations twice, though?
    make_map_ts: Timestamp,
}

impl Map {
    pub fn new(make_map_ts: Timestamp) -> Self {
        Self { make_map_ts }
    }
}
