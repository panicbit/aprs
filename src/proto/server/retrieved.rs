use std::sync::Arc;

use fnv::FnvHashMap;
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug)]
pub struct Retrieved {
    pub keys: FnvHashMap<String, Arc<Value>>,
}
