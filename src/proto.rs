pub mod client;
pub mod common;
pub mod server;

mod u128_uuid {
    use serde::{Deserialize, Deserializer, de};
    use serde_json::Number;
    use uuid::Uuid;

    pub fn deserialize<'de, D>(de: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let uuid = Number::deserialize(de)?;
        let uuid = uuid
            .as_u128()
            .ok_or_else(|| de::Error::custom("invalid u128 for uuid"))?;
        let uuid = Uuid::from_u128(uuid);

        Ok(uuid)
    }
}
