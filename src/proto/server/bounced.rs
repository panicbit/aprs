use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::game::SlotId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Bounced {
    pub games: Vec<String>,
    pub slots: Vec<SlotId>,
    pub tags: Vec<String>,
    pub data: Value,
}
