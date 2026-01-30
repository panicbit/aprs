use aprs_value::Value;
use serde::{Deserialize, Serialize};

use crate::primitives::SlotId;

#[derive(Serialize, Deserialize, Debug)]
pub struct SetReply {
    pub key: String,
    pub value: Value,
    pub original_value: Value,
    pub slot: SlotId,
}
