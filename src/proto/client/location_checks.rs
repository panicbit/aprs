use fnv::FnvHashSet;
use serde::Deserialize;

use crate::game::LocationId;

#[derive(Deserialize, Debug)]
pub struct LocationChecks {
    pub locations: FnvHashSet<LocationId>,
}
