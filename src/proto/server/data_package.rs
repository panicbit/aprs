use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Serialize;

use crate::game::HashedGameData;

#[derive(Serialize, Debug)]
pub struct DataPackage {
    pub data: Arc<BTreeMap<String, HashedGameData>>,
}
