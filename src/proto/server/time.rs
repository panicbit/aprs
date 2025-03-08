use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Copy, Clone)]
pub struct Time(DateTime<Utc>);

impl Time {
    pub fn now() -> Self {
        Self(Utc::now())
    }
}

impl Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let time = self.0.timestamp() as f64;

        serializer.serialize_f64(time)
    }
}
