use aprs_proto::primitives::ItemId;
use aprs_utils::collections::map;

use crate::LazyMap;

pub static ITEM_TABLE: LazyMap<&str, ItemId> = LazyMap::new(|| {
    map! {
        "Trinket 01" => ItemId(2515000),
        "Trinket 02" => ItemId(2515001),
        "Trinket 03" => ItemId(2515002),
        "Trinket 04" => ItemId(2515003),
        "Trinket 05" => ItemId(2515004),
        "Trinket 06" => ItemId(2515005),
        "Trinket 07" => ItemId(2515006),
        "Trinket 08" => ItemId(2515007),
        "Trinket 09" => ItemId(2515008),
        "Trinket 10" => ItemId(2515009),
        "Trinket 11" => ItemId(2515010),
        "Trinket 12" => ItemId(2515011),
        "Trinket 13" => ItemId(2515012),
        "Trinket 14" => ItemId(2515013),
        "Trinket 15" => ItemId(2515014),
        "Trinket 16" => ItemId(2515015),
        "Trinket 17" => ItemId(2515016),
        "Trinket 18" => ItemId(2515017),
        "Trinket 19" => ItemId(2515018),
        "Trinket 20" => ItemId(2515019),
    }
});
