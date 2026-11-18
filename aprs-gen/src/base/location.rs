use aprs_proto::primitives::SlotId;

use crate::base::{GameName, Item, LocationName};

pub struct Location {
    pub game: GameName,
    pub player: SlotId,
    pub name: LocationName,
    pub address: Option<u32>,
    pub item: Option<Item>,
}
