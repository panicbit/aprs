use serde::{Deserialize, Deserializer};
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
