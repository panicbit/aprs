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
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WeirdUuid {
            Uuid(Uuid),
            /// The python does not send a hex string uuid, but a number
            Number(Number),
        }

        let uuid = WeirdUuid::deserialize(de)?;

        match uuid {
            WeirdUuid::Uuid(uuid) => Ok(uuid),
            WeirdUuid::Number(number) => {
                let uuid = number
                    .as_u128()
                    .ok_or_else(|| de::Error::custom("invalid u128 for uuid"))?;

                Ok(Uuid::from_u128(uuid))
            }
        }
    }
}
