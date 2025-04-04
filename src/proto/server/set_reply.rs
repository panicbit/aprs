use serde::{Deserialize, Serialize};

use crate::game::SlotId;
use crate::pickle::Value;
use crate::pickle::value::Str;

#[derive(Serialize, Deserialize, Debug)]
pub struct SetReply {
    pub key: Str,
    pub value: Value,
    pub original_value: Value,
    pub slot: SlotId,
}
