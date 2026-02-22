use serde::{Deserialize, Serialize, de};

#[derive(Serialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotId(pub i64);

impl SlotId {
    pub const SERVER: SlotId = SlotId(0);

    pub fn is_server(&self) -> bool {
        self.0 == 0
    }
}

impl<'de> Deserialize<'de> for SlotId {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum IntOrString {
            Int(i64),
            String(String),
        }

        Ok(match IntOrString::deserialize(de)? {
            IntOrString::Int(n) => SlotId(n),
            IntOrString::String(n) => n.parse::<i64>().map(SlotId).map_err(de::Error::custom)?,
        })
    }
}
