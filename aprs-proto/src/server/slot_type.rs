use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(transparent)]
pub struct SlotType(u32);

bitflags! {
    impl SlotType: u32 {
        const Player = 0b01;
        const Group = 0b10;
    }
}

impl SlotType {
    pub fn is_spectator(&self) -> bool {
        self.is_empty()
    }

    pub fn is_player(&self) -> bool {
        self.contains(SlotType::Player)
    }

    pub fn is_group(&self) -> bool {
        self.contains(SlotType::Group)
    }
}
