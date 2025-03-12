use serde::Serialize;

use crate::game::{ItemId, LocationId, SlotId};

#[derive(Serialize, Clone, Debug)]
pub struct NetworkItem {
    pub item: ItemId,
    pub location: LocationId,
    pub player: SlotId,
    pub flags: u64,
}
