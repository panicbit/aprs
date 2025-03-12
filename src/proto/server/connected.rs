use std::collections::BTreeMap;
use std::sync::Arc;

use fnv::FnvHashSet;
use serde::Serialize;

use crate::game::{LocationId, NetworkSlot, SlotId, TeamId};
use crate::proto::server::NetworkPlayer;

#[derive(Serialize, Clone, Debug)]
pub struct Connected {
    pub team: TeamId,
    pub slot: SlotId,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: FnvHashSet<LocationId>,
    pub checked_locations: FnvHashSet<LocationId>,
    pub slot_data: Arc<serde_json::Value>,
    pub slot_info: Arc<BTreeMap<SlotId, NetworkSlot>>,
    pub hint_points: u32,
}
