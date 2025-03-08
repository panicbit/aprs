use archipelago_core::game::SlotId;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct Bounced {
    pub games: Vec<String>,
    pub slots: Vec<SlotId>,
    pub tags: Vec<String>,
    pub data: Value,
}
