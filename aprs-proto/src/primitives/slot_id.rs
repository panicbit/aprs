use serde::{Deserialize, Serialize};

use crate::deserialize::i64_or_string;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotId(#[serde(deserialize_with = "i64_or_string")] pub i64);

impl SlotId {
    pub const SERVER: SlotId = SlotId(0);

    pub fn is_server(&self) -> bool {
        self.0 == 0
    }
}
