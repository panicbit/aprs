use bitflags::bitflags;
use uuid::Uuid;

use crate::game::SlotName;
use crate::proto::client::Deserialize;
use crate::proto::common::NetworkVersion;
use crate::proto::u128_uuid;

#[derive(Deserialize, Clone, Debug)]
pub struct Connect {
    pub password: Option<String>,
    pub game: String,
    pub name: SlotName,
    // parse either number OR string
    #[serde(with = "u128_uuid")]
    pub uuid: Uuid,
    pub version: NetworkVersion,
    pub items_handling: ItemsHandling,
    pub tags: Vec<String>,
    pub slot_data: bool,
}

#[derive(Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ItemsHandling(u8);

bitflags! {
    impl ItemsHandling: u8 {
        const Remote = 0b001;
        const OwnWorld = 0b010;
        const StartingInventory = 0b100;
    }
}

impl ItemsHandling {
    pub fn is_no_items(&self) -> bool {
        self.is_empty()
    }

    pub fn is_remote(&self) -> bool {
        self.contains(Self::Remote)
    }

    pub fn is_own_world(&self) -> bool {
        self.contains(Self::OwnWorld)
    }

    pub fn is_starting_inventory(&self) -> bool {
        self.contains(Self::StartingInventory)
    }
}
