use std::time::{SystemTime, UNIX_EPOCH};

use super::types::TimestampMillis;

pub struct Util;

impl Util {
    pub fn current_time_millis() -> TimestampMillis {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}
