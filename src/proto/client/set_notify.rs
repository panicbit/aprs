use fnv::FnvHashSet;
use serde::Deserialize;

use crate::pickle::value::Str;

#[derive(Deserialize, Debug)]
pub struct SetNotify {
    pub keys: FnvHashSet<Str>,
}
