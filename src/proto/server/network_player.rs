use serde::Serialize;

use crate::game::{SlotId, SlotName, TeamId};

#[derive(Serialize, Clone, Debug)]
pub struct NetworkPlayer {
    pub team: TeamId,
    pub slot: SlotId,
    pub alias: String,
    pub name: SlotName,
}
