use serde::{Deserialize, Deserializer, de};
use serde_json::Number;

pub fn u128_uuid<'de, D>(de: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WeirdUuid {
        String(String),
        /// The python client does not send a hex formatted uuid, but a number
        Number(Number),
    }

    let uuid = WeirdUuid::deserialize(de)?;

    match uuid {
        WeirdUuid::String(string) => Ok(string),
        WeirdUuid::Number(number) => Ok(number.to_string()),
    }
}

pub fn i64_or_string<'de, D>(de: D) -> Result<i64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum InternalValue {
        Int(i64),
        String(String),
    }

    let value = InternalValue::deserialize(de)?;

    match value {
        InternalValue::Int(n) => Ok(n),
        InternalValue::String(s) => s.parse::<i64>().map_err(de::Error::custom),
    }
}
