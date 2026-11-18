use std::sync::{Arc, LazyLock};

use aprs_proto::primitives::SlotId;
use aprs_utils::collections::Set;

use crate::Map;
use crate::base::GameName;
use crate::world::{NewWorld, World};

mod v6;
use v6::V6;

pub struct WorldFactory {
    pub new_player: fn(SlotId) -> Arc<dyn World>,
    pub new_group: fn(SlotId, &Set<SlotId>) -> Arc<dyn World>,
}

impl WorldFactory {
    pub fn new_player(&self, player: SlotId) -> Arc<dyn World> {
        (self.new_player)(player)
    }

    pub fn new_group(&self, group: SlotId, players: &Set<SlotId>) -> Arc<dyn World> {
        (self.new_group)(group, players)
    }
}

fn register<W: NewWorld>() -> (GameName, WorldFactory) {
    let factory = WorldFactory {
        new_player: |player| Arc::new(W::new_player(player)),
        new_group: |group, players| Arc::new(W::new_group(group, players)),
    };

    (W::NAME.into(), factory)
}

pub static WORLD_FACTORIES: LazyLock<Map<GameName, WorldFactory>> = LazyLock::new(|| {
    [
        register::<V6>(), //
    ]
    .into_iter()
    .collect()
});
