use serde::Deserialize;

use crate::primitives::GameName;

#[derive(Deserialize, Debug)]
pub struct GetDataPackage {
    pub games: Vec<GameName>,
}
