use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Retrieved<V> {
    pub keys: FnvHashMap<String, V>,
}
