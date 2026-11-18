use std::sync::LazyLock;

use aprs_proto::primitives::LocationId;
use aprs_utils::collections::map;

use crate::Map;

pub static LOCATION_TABLE: LazyLock<Map<&str, LocationId>> = LazyLock::new(|| {
    map! {
        "It's a Secret to Nobody" => LocationId(2515000),
        "Trench Warfare" => LocationId(2515001),
        "One Way Room" => LocationId(2515002),
        "You Just Keep Coming Back" => LocationId(2515003),
        "Clarion Call" => LocationId(2515004),
        "Doing things the hard way" => LocationId(2515005),
        "Prize for the Reckless" => LocationId(2515006),
        "The Tower 1" => LocationId(2515007),
        "The Tower 2" => LocationId(2515008),
        "Young Man, It's Worth the Challenge" => LocationId(2515009),
        "The Tantalizing Trinket" => LocationId(2515010),
        "Purest Unobtainium" => LocationId(2515011),
        "Edge Games" => LocationId(2515012),
        "Overworld (Pipe-shaped Segment)" => LocationId(2515013),
        "Overworld (Outside Entanglement Generator)" => LocationId(2515014),
        "Overworld (Left of Ship)" => LocationId(2515015),
        "Overworld (Square Room)" => LocationId(2515016),
        "Overworld (Sad Elephant)" => LocationId(2515017),
        "NPC Trinket" => LocationId(2515018),
        "V" => LocationId(2515019),
    }
});
