use aprs_value::Value;
use serde::{Deserialize, Serialize};

use crate::client;
use crate::primitives::SlotId;

#[derive(Serialize, Deserialize, Debug)]
pub struct Bounced {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub games: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<SlotId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl From<client::Bounce> for Bounced {
    fn from(value: client::Bounce) -> Self {
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
