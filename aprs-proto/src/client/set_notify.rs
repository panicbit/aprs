use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SetNotify {
    pub keys: FnvHashSet<String>,
}
