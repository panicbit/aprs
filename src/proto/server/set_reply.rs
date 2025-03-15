use serde::Serialize;

use crate::game::SlotId;
use crate::pickle::Value;
use crate::pickle::value::Str;

#[derive(Serialize, Debug)]
pub struct SetReply {
    pub key: Str,
    pub value: Value,
    pub original_value: Value,
    pub slot: SlotId,
}
