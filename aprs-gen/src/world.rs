use aprs_proto::primitives::{ItemId, LocationId, SlotId};
use aprs_utils::collections::Set;

use crate::StaticMap;

pub trait NewWorld {
    const NAME: &'static str;

    type PlayerWorld: World;
    type GroupWorld: World;

    fn new_player(player: SlotId) -> Self::PlayerWorld;
    fn new_group(group: SlotId, players: &Set<SlotId>) -> Self::GroupWorld;
}

pub trait NewPlayerWorld: World {
    const NAME: &'static str;

    fn new_player_world(player: SlotId) -> Self;
}

impl<NPW: NewPlayerWorld> NewWorld for NPW {
    const NAME: &'static str = NPW::NAME;

    type PlayerWorld = NPW;
    type GroupWorld = NPW;

    fn new_player(player: SlotId) -> Self::PlayerWorld {
        NPW::new_player_world(player)
    }

    fn new_group(group: SlotId, _players: &Set<SlotId>) -> Self::GroupWorld {
        // TODO: set item accessibility option to `items`
        NPW::new_player_world(group)
    }
}

pub trait World: 'static {
    fn item_name_to_id(&self) -> StaticMap<&str, ItemId>;
    fn location_name_to_id(&self) -> StaticMap<&str, LocationId>;
    fn create_regions(&self);
}
