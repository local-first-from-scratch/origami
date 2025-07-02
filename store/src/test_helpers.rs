use uuid::Uuid;

proptest::prop_compose! {
    pub fn uuid()(u: u128) -> Uuid {
        Uuid::from_u128(u)
    }
}
