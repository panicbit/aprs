use fnv::FnvHashSet;
use serde::Serialize;

use crate::game::LocationId;

#[derive(Serialize, Debug, Default)]
pub struct RoomUpdate {
    // TODO: implement more fields
    #[serde(skip_serializing_if = "FnvHashSet::is_empty")]
    pub checked_locations: FnvHashSet<LocationId>,
}

impl RoomUpdate {
    pub fn checked_locations(checked_locations: FnvHashSet<LocationId>) -> Self {
        Self { checked_locations }
    }
}
