use super::Clock;

pub struct SystemTime;

impl Clock for SystemTime {
    fn unix_timestamp() -> u32 {
        // The cast here is OK; we're only using this for testing.
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32
    }
}
