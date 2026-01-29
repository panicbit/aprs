use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotId(pub i64);

impl SlotId {
    pub const SERVER: SlotId = SlotId(0);

    pub fn is_server(&self) -> bool {
        self.0 == 0
    }
}
