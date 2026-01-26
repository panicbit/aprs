use serde::{Deserialize, Serialize};

use crate::server::NetworkItem;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReceivedItems {
    pub index: usize,
    pub items: Vec<NetworkItem>,
}
