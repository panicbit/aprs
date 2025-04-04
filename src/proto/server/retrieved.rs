use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::pickle::Value;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Retrieved {
    pub keys: FnvHashMap<String, Value>,
}
