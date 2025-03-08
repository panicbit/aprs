use archipelago_core::game::{SlotId, SlotName, TeamId};
use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct NetworkPlayer {
    pub team: TeamId,
    pub slot: SlotId,
    pub alias: String,
    pub name: SlotName,
}
