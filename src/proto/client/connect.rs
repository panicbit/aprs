use uuid::Uuid;

use crate::game::SlotName;
use crate::proto::client::{Deserialize, NetworkVersion};
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
    pub items_handling: u8,
    pub tags: Vec<String>,
    pub slot_data: bool,
}
