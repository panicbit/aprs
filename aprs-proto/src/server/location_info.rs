use serde::{Deserialize, Serialize};

use crate::server::NetworkItem;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocationInfo {
    pub locations: Vec<NetworkItem>,
}
