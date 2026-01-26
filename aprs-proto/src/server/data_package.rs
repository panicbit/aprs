use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::server::HashedGameData;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackage {
    pub data: DataPackageData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackageData {
    pub games: Arc<BTreeMap<String, HashedGameData>>,
}
