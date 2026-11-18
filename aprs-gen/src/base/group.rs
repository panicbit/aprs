use aprs_proto::primitives::{ItemId, SlotId};
use aprs_utils::collections::{Map, Set};

use crate::base::{GameName, SlotName};

pub struct Group {
    pub name: SlotName,
    pub game: GameName,
    // world: World,
    pub players: Set<SlotId>,
    pub item_pool: Set<String>,
    pub replacement_items: Map<ItemId, Option<String>>,
    pub local_items: Set<String>,
    pub non_local_items: Set<String>,
    pub link_replacement: bool,
}

impl Group {
    pub fn new(name: SlotName, game: GameName, players: Set<SlotId>) -> Self {
        Self {
            name,
            game,
            players,
            item_pool: Set::default(),
            replacement_items: Map::default(),
            local_items: Set::default(),
            non_local_items: Set::default(),
            link_replacement: false,
        }
    }
}
