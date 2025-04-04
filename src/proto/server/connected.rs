use std::collections::BTreeMap;
use std::sync::Arc;

use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

use crate::game::{LocationId, NetworkSlot, SlotId, TeamId};
use crate::pickle::Value;
use crate::proto::server::NetworkPlayer;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connected {
    pub team: TeamId,
    pub slot: SlotId,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: FnvHashSet<LocationId>,
    pub checked_locations: FnvHashSet<LocationId>,
    pub slot_data: Value,
    pub slot_info: Arc<BTreeMap<SlotId, NetworkSlot>>,
    pub hint_points: u32,
}
