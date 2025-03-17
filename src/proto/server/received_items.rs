use serde::Serialize;

use crate::proto::server::NetworkItem;

#[derive(Serialize, Debug)]
pub struct ReceivedItems {
    pub index: usize,
    pub items: Vec<NetworkItem>,
}
