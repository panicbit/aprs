use serde::{Deserialize, Serialize};

use crate::primitives::{Map, ItemId, LocationId};

// note: checksums of game data sent over the wire can't be verified,
//       because the game data sent over the wire is missing certain fields

#[derive(Deserialize, Serialize, Debug)]
pub struct GameData {
    pub checksum: String,
    pub item_name_to_id: Map<String, ItemId>,
    pub location_name_to_id: Map<String, LocationId>,
}
