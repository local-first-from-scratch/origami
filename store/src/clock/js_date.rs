use super::Clock;

pub struct JsDate;

impl Clock for JsDate {
    fn unix_timestamp() -> u32 {
        let ms = js_sys::Date::now();

        // TODO: should we do this more safely? Has the possibility of
        // truncating the date.
        (ms / 1000.0).round() as u32
    }
}
