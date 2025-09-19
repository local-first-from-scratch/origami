pub mod js_date;
pub mod system_time;

pub trait Clock {
    fn unix_timestamp() -> u32;
}
