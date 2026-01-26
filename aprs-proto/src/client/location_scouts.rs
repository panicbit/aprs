use serde::Deserialize;

use crate::primitives::LocationId;

#[derive(Deserialize, Debug)]
pub struct LocationScouts {
    pub locations: Vec<LocationId>,
    pub create_as_hint: i32,
}
