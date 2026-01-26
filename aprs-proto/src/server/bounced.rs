use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::client;
use crate::primitives::SlotId;

#[derive(Serialize, Deserialize, Debug)]
pub struct Bounced<V> {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub games: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<V>,
}

impl<V: DeserializeOwned> From<client::Bounce<V>> for Bounced<V> {
    fn from(value: client::Bounce<V>) -> Self {
        let client::Bounce {
            games,
            slots,
            tags,
            data,
        } = value;

        Self {
            games,
            slots,
            tags,
            data,
        }
    }
}
