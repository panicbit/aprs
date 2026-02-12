use fnv::FnvHashSet;
use serde::{Deserialize, Serialize};

use crate::primitives::LocationId;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocationChecks {
    pub locations: FnvHashSet<LocationId>,
}
