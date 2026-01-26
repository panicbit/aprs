use serde::{Deserialize, Serialize};

use crate::primitives::{SlotId, SlotName, TeamId};

#[derive(Deserialize, Clone, Debug)]
pub struct NetworkPlayer {
    pub team: TeamId,
    pub slot: SlotId,
    pub alias: String,
    pub name: SlotName,
}

impl Serialize for NetworkPlayer {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let NetworkPlayer {
            team,
            slot,
            ref alias,
            ref name,
        } = *self;

        // The python client expects a "class" field in the json serialization
        #[derive(Serialize)]
        #[serde(tag = "class", rename = "NetworkPlayer")]
        struct PythonNetworkPlayer<'a> {
            pub team: TeamId,
            pub slot: SlotId,
            pub alias: &'a String,
            pub name: &'a SlotName,
        }

        PythonNetworkPlayer {
            team,
            slot,
            alias,
            name,
        }
        .serialize(ser)
    }
}
