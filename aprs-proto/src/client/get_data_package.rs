use serde::{Deserialize, Serialize};

use crate::primitives::GameName;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDataPackage {
    pub games: Vec<GameName>,
}
