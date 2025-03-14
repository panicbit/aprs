use serde::Serialize;

use crate::game::SlotId;
use crate::pickle::Value;

#[derive(Serialize, Debug)]
pub struct SetReply {
    pub key: String,
    pub value: Value,
    pub original_value: Value,
    pub slot: SlotId,
}
