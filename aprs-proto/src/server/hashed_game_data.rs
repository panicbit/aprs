use std::ops;

use serde::{Deserialize, Serialize};

use crate::server::GameData;

// note: checksums of game data sent over the wire can't be verified,
//       because the game data sent over the wire is missing certain fields

#[derive(Deserialize, Serialize, Debug)]
pub struct HashedGameData {
    pub checksum: String,
    pub game_data: GameData,
}

impl ops::Deref for HashedGameData {
    type Target = GameData;

    fn deref(&self) -> &Self::Target {
        &self.game_data
    }
}

impl ops::DerefMut for HashedGameData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game_data
    }
}
