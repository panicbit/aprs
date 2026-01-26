use serde::{Deserialize, Serialize};

use crate::primitives::SlotName;
use crate::server::SlotType;

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSlot {
    pub name: SlotName,
    pub game: String,
    pub r#type: SlotType,
    // TODO: implement for completeness some day maybe
    // https://github.com/ArchipelagoMW/Archipelago/blob/e00467c2a299623f630d5a3e68f35bc56ccaa8aa/NetUtils.py#L86
    pub group_members: serde_json::Value,
}

impl Serialize for NetworkSlot {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let NetworkSlot {
            name,
            game,
            r#type,
            group_members,
        } = &self;

        // The python client expects a "class" field in the json serialization
        #[derive(Serialize)]
        #[serde(tag = "class", rename = "NetworkSlot")]
        struct PythonNetworkSlot<'a> {
            pub name: &'a str,
            pub game: &'a str,
            pub r#type: &'a SlotType,
            // TODO: implement for completeness some day maybe
            // https://github.com/ArchipelagoMW/Archipelago/blob/e00467c2a299623f630d5a3e68f35bc56ccaa8aa/NetUtils.py#L86
            pub group_members: &'a serde_json::Value,
        }

        PythonNetworkSlot {
            name,
            game,
            r#type,
            group_members,
        }
        .serialize(ser)
    }
}
