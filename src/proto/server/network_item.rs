use serde::Serialize;

use crate::game::{ItemId, LocationId, SlotId};

#[derive(Copy, Clone, Debug, Hash)]
pub struct NetworkItem {
    pub item: ItemId,
    pub location: LocationId,
    pub player: SlotId,
    pub flags: u64,
}

impl Serialize for NetworkItem {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let NetworkItem {
            item,
            location,
            player,
            flags,
        } = *self;

        // The python client expects a "class" field in the json serialization
        #[derive(Serialize)]
        #[serde(tag = "class", rename = "NetworkItem")]
        struct PythonNetworkItem {
            pub item: ItemId,
            pub location: LocationId,
            pub player: SlotId,
            pub flags: u64,
        }

        PythonNetworkItem {
            item,
            location,
            player,
            flags,
        }
        .serialize(ser)
    }
}

impl NetworkItem {
    pub fn with_player(mut self, player: SlotId) -> Self {
        self.player = player;
        self
    }
}
