use aprs_proto::primitives::SlotId;

mod multi_world;
pub use multi_world::MultiWorld;

mod region;
pub use region::Region;

mod region_manager;
pub use region_manager::RegionManager;

mod entrance;
pub use entrance::Entrance;

mod location;
pub use location::Location;

mod item;
pub use item::Item;

mod seed;
pub use seed::Seed;

mod random;
pub use random::Random;

mod spoiler;
pub use spoiler::Spoiler;

mod plando_item_block;
pub use plando_item_block::PlandoItemBlock;

mod group;
pub use group::Group;

mod strings;
pub use strings::*;

mod collection_state;
pub use collection_state::CollectionState;

fn player_ids(players: usize) -> impl Iterator<Item = SlotId> {
    (1..=players).map(|players| players as i64).map(SlotId)
}
