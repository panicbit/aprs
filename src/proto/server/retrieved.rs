use fnv::FnvHashMap;
use serde::Serialize;

use crate::pickle::Value;

#[derive(Serialize, Clone, Debug)]
pub struct Retrieved {
    pub keys: FnvHashMap<String, Value>,
}
