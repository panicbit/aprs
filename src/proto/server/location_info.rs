use serde::Serialize;

use crate::proto::server::NetworkItem;

#[derive(Serialize, Clone, Debug)]
pub struct LocationInfo {
    pub locations: Vec<NetworkItem>,
}
