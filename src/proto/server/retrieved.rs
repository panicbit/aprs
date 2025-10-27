use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::pickle::Value;
use crate::pickle::value::storage;

type S = storage::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Retrieved {
    pub keys: FnvHashMap<String, Value<S>>,
}
