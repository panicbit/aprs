use fnv::FnvHashSet;
use serde::Deserialize;

use crate::pickle::value::{Str, storage};

type S = storage::Arc;

#[derive(Deserialize, Debug)]
pub struct SetNotify {
    pub keys: FnvHashSet<Str<S>>,
}
