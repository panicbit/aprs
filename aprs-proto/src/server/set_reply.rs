use serde::{Deserialize, Serialize};

use crate::primitives::SlotId;

#[derive(Serialize, Deserialize, Debug)]
pub struct SetReply<V> {
    pub key: String,
    pub value: V,
    pub original_value: V,
    pub slot: SlotId,
}
