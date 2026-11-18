use aprs_utils::SharedStr;

mod brand {
    pub struct SlotName(());
    pub struct GameName(());
    pub struct RegionName(());
    pub struct EntranceName(());
    pub struct LocationName(());
    pub struct ItemName(());
}

pub type SlotName = SharedStr<brand::SlotName>;
pub type GameName = SharedStr<brand::GameName>;
pub type RegionName = SharedStr<brand::RegionName>;
pub type EntranceName = SharedStr<brand::EntranceName>;
pub type LocationName = SharedStr<brand::LocationName>;
pub type ItemName = SharedStr<brand::ItemName>;
