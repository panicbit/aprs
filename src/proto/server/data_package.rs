use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::game::HashedGameData;

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackage {
    pub data: DataPackageData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPackageData {
    pub games: Arc<BTreeMap<String, HashedGameData>>,
}
