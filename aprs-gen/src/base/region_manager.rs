use std::ops;

use aprs_proto::primitives::SlotId;
use aprs_utils::collections::Map;

use crate::base::region::Region;
use crate::base::{Entrance, EntranceName, Location, LocationName, RegionName, player_ids};

pub struct RegionManager {
    region_cache: Map<SlotId, Map<RegionName, Region>>,
    entrance_cache: Map<SlotId, Map<EntranceName, Entrance>>,
    location_cache: Map<SlotId, Map<LocationName, Location>>,
}

impl RegionManager {
    pub fn new(players: usize) -> Self {
        Self {
            region_cache: player_ids(players)
                .map(|slot_id| (slot_id, Map::default()))
                .collect(),
            entrance_cache: player_ids(players)
                .map(|slot_id| (slot_id, Map::default()))
                .collect(),
            location_cache: player_ids(players)
                .map(|slot_id| (slot_id, Map::default()))
                .collect(),
        }
    }

    pub fn regions(&self, player: SlotId) -> Option<&Map<RegionName, Region>> {
        self.region_cache.get(&player)
    }

    pub fn entrances(&self, player: SlotId) -> Option<&Map<EntranceName, Entrance>> {
        self.entrance_cache.get(&player)
    }

    pub fn locations(&self, player: SlotId) -> Option<&Map<LocationName, Location>> {
        self.location_cache.get(&player)
    }

    pub fn all_regions(&self) -> impl Iterator<Item = &Region> {
        self.region_cache
            .values()
            .flat_map(|regions| regions.values())
    }

    pub fn all_entrances(&self) -> impl Iterator<Item = &Entrance> {
        self.entrance_cache
            .values()
            .flat_map(|entrances| entrances.values())
    }

    pub fn all_locations(&self) -> impl Iterator<Item = &Location> {
        self.location_cache
            .values()
            .flat_map(|location| location.values())
    }

    pub fn add_region(&mut self, region: Region) {
        // TODO: assert region does not already exist in cache
        let Some(regions) = self.region_cache.get_mut(&region.player) else {
            // TODO: report invalid player, bail
            return;
        };

        regions.insert(region.name.clone(), region);
    }

    pub fn add_regions(&mut self, regions: impl IntoIterator<Item = Region>) {
        for region in regions {
            self.add_region(region);
        }
    }

    pub fn add_group(&mut self, group_id: SlotId) {
        self.region_cache.insert(group_id, Map::default());
        self.entrance_cache.insert(group_id, Map::default());
        self.location_cache.insert(group_id, Map::default());
    }

    pub fn len(&self) -> usize {
        self.region_cache
            .values()
            .map(|regions| regions.len())
            .sum()
    }
}

impl ops::AddAssign<Region> for RegionManager {
    fn add_assign(&mut self, region: Region) {
        self.add_region(region);
    }
}

impl ops::AddAssign<Vec<Region>> for RegionManager {
    fn add_assign(&mut self, regions: Vec<Region>) {
        self.add_regions(regions);
    }
}

impl<const N: usize> ops::AddAssign<[Region; N]> for RegionManager {
    fn add_assign(&mut self, regions: [Region; N]) {
        self.add_regions(regions);
    }
}
