use serde::Serialize;
use serde_json::Value;

use crate::game::SlotId;

#[derive(Serialize, Debug)]
pub struct SetReply {
    pub key: String,
    pub value: Value,
    pub original_value: Value,
    pub slot: SlotId,
}
