use serde::Deserialize;

use crate::proto::server::GameName;

#[derive(Deserialize, Debug)]
pub struct GetDataPackage {
    pub games: Vec<GameName>,
}
