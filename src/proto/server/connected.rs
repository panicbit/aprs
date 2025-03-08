use std::collections::BTreeMap;

use archipelago_core::game::{LocationId, NetworkSlot, SlotId, TeamId};
use serde::Serialize;

use crate::proto::server::NetworkPlayer;

#[derive(Serialize, Clone, Debug)]
pub struct Connected {
    pub team: TeamId,
    pub slot: SlotId,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: Vec<LocationId>,
    pub checked_locations: Vec<LocationId>,
    pub slot_data: serde_json::Value,
    pub slot_info: BTreeMap<SlotId, NetworkSlot>,
    pub hint_points: u32,
}
