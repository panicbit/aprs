use aprs_proto::primitives::SlotId;

use crate::base::ItemName;

pub struct Item {
    pub player: SlotId,
    pub name: ItemName,
}
