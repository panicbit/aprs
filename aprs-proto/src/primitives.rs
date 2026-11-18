mod connect_name;
pub use connect_name::ConnectName;

mod game_name;
pub use game_name::GameName;

mod item_id;
pub use item_id::ItemId;

mod location_id;
pub use location_id::LocationId;

mod slot_id;
pub use slot_id::SlotId;

mod slot_name;
pub use slot_name::SlotName;

mod team_id;
pub use team_id::TeamId;

pub type Hasher = aprs_utils::hasher::FxHasher;
pub type Map<K, V> = aprs_utils::collections::FxIndexMap<K, V>;
pub type Set<K> = aprs_utils::collections::FxIndexSet<K>;
