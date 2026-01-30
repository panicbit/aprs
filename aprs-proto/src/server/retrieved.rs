use aprs_value::Value;
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Retrieved {
    pub keys: FnvHashMap<String, Value>,
}
