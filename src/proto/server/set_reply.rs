use serde::{Deserialize, Serialize};

use crate::game::SlotId;
use crate::pickle::Value;
use crate::pickle::value::{Str, storage};

type S = storage::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct SetReply {
    pub key: Str<S>,
    pub value: Value<S>,
    pub original_value: Value<S>,
    pub slot: SlotId,
}
