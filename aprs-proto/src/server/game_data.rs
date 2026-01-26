use serde::{Deserialize, Serialize};

use crate::primitives::{FnvIndexMap, ItemId, LocationId};

// note: checksums of game data sent over the wire can't be verified,
//       because the game data sent over the wire is missing certain fields

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameData {
    pub item_name_to_id: FnvIndexMap<String, ItemId>,
    pub location_name_to_id: FnvIndexMap<String, LocationId>,
}
