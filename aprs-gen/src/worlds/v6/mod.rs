use aprs_proto::primitives::{ItemId, LocationId, SlotId};

use crate::StaticMap;

mod items;
mod locations;
mod options;

pub struct V6 {}

impl crate::NewPlayerWorld for V6 {
    const NAME: &'static str = "VVVVVV";

    fn new_player_world(player: SlotId) -> Self {
        Self {}
    }
}

impl crate::World for V6 {
    fn item_name_to_id(&self) -> StaticMap<&str, ItemId> {
        &items::ITEM_TABLE
    }

    fn location_name_to_id(&self) -> StaticMap<&str, LocationId> {
        &locations::LOCATION_TABLE
    }

    fn create_regions(&self) {
        todo!()
    }
}
