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
        const OwnWorldOnly = 0b010;
        const StartingInventoryOnly = 0b100;
    }
}

impl ItemsHandling {
    pub fn is_no_items(&self) -> bool {
        self.is_empty()
    }

    pub fn is_remote(&self) -> bool {
        self.contains(Self::Remote)
    }

    pub fn is_own_world_only(&self) -> bool {
        self.contains(Self::OwnWorldOnly)
    }

    pub fn is_starting_inventory_only(&self) -> bool {
        self.contains(Self::StartingInventoryOnly)
    }
}
