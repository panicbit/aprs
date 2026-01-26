use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de};

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
        // TODO: handle subsec precision
        let time = self.0.timestamp() as f64;

        serializer.serialize_f64(time)
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // TODO: handle subsec precision
        let time = f64::deserialize(de)?;
        let time = time as i64;
        let time = DateTime::from_timestamp(time, 0)
            .ok_or_else(|| de::Error::custom("invalid timestamp"))?;

        Ok(Time(time))
    }
}
