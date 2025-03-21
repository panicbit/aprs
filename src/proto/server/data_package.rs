use serde::Serialize;

use crate::game::GameData;

#[derive(Serialize, Debug)]
pub struct DataPackage {
    pub data: GameData,
}
