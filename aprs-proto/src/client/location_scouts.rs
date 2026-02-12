use serde::{Deserialize, Serialize};

use crate::primitives::LocationId;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocationScouts {
    pub locations: Vec<LocationId>,
    pub create_as_hint: i32,
}
