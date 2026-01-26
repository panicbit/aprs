use std::collections::BTreeMap;
use std::sync::Arc;

use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

use crate::primitives::{LocationId, SlotId, TeamId};
use crate::server::{NetworkPlayer, NetworkSlot};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connected<V> {
    pub team: TeamId,
    pub slot: SlotId,
    pub players: Vec<NetworkPlayer>,
    pub missing_locations: FnvHashSet<LocationId>,
    pub checked_locations: FnvHashSet<LocationId>,
    pub slot_data: V,
    pub slot_info: Arc<BTreeMap<SlotId, NetworkSlot>>,
    pub hint_points: u32,
}
