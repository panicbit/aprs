use fnv::FnvHashSet;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SetNotify {
    pub keys: FnvHashSet<String>,
}
