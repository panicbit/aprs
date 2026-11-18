use std::sync::Arc;

use aprs_proto::primitives::SlotId;

use crate::base::RegionName;

#[derive(PartialEq, Eq)]
pub struct Region {
    pub name: RegionName,
    pub player: SlotId,
}
