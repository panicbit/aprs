use std::borrow::Cow;
use std::sync::Arc;

use aprs_proto::primitives::{ItemId, SlotId};
use aprs_proto::server::SlotType;
use aprs_utils::IteratorExt;
use aprs_utils::collections::{Map, Set};
use bitflags::bitflags;
use either::Either::{Left, Right};

use crate::base::{
    CollectionState, Entrance, EntranceName, GameName, Group, Item, ItemName, Location,
    LocationName, PlandoItemBlock, Random, Region, RegionManager, RegionName, Seed, SlotName,
    Spoiler, player_ids,
};
use crate::utils;
use crate::world::World;
use crate::worlds::WORLD_FACTORIES;

// TODO: Can the maps with SlotId keys be aggregated into a single map?
// TODO: Can maps with SlotId keys be replaced with a Vec?

pub struct MultiWorld {
    random: Random,
    num_players: usize,
    player_types: Map<SlotId, SlotType>,
    player_names: Map<SlotId, SlotName>,
    algorithm: Algorithm,
    groups: Map<SlotId, Group>,
    region_manager: RegionManager,
    item_pool: Vec<Item>,
    seed: Seed,
    seed_name: String,
    precollected_items: Map<SlotId, Vec<Item>>,
    required_locations: Vec<Location>,
    custom: bool,
    // TODO: remove `custom_item_array`? only used by AlttP.
    custom_item_array: Vec<Item>,
    // TODO: remove `shuffle_ganon`? only used by AlttP.
    shuffle_ganon: bool,
    spoiler: Spoiler,
    early_items: Map<SlotId, Map<Arc<String>, usize>>,
    local_early_items: Map<SlotId, Map<Arc<String>, usize>>,
    indirect_connections: Map<Arc<Region>, Set<Entrance>>,
    // TODO: create StartInventoryPool option
    start_inventory_from_pool: (),
    plando_item_blocks: Map<SlotId, Vec<PlandoItemBlock>>,
    worlds: Map<SlotId, Arc<dyn World>>,
    plando_options: PlandoOptions,
    game: Map<SlotId, GameName>,
    completion_condition: Map<SlotId, fn(State) -> bool>,
    is_race: bool,
    collection_state: CollectionState,
}

impl MultiWorld {
    pub fn new(num_players: usize) -> Self {
        let seed = Seed::new_random();

        Self {
            random: Random::seeded(seed),
            num_players,
            player_names: Map::default(),
            player_types: player_ids(num_players)
                .with_value(|_| SlotType::Player)
                .collect(),
            algorithm: Algorithm::Balanced,
            groups: Map::default(),
            region_manager: RegionManager::new(num_players),
            item_pool: Vec::new(),
            seed,
            seed_name: "Unavailable".into(),
            precollected_items: player_ids(num_players) //
                .with_value(|_| Vec::new())
                .collect(),
            required_locations: Vec::new(),
            custom: false,
            custom_item_array: Vec::new(),
            shuffle_ganon: true,
            spoiler: Spoiler::new(),
            early_items: player_ids(num_players) //
                .with_value(|_| Map::default())
                .collect(),
            local_early_items: player_ids(num_players) //
                .with_value(|_| Map::default())
                .collect(),
            indirect_connections: Map::default(),
            start_inventory_from_pool: (),
            plando_item_blocks: player_ids(num_players) //
                .with_value(|_| Vec::new())
                .collect(),
            worlds: Map::default(),
            plando_options: PlandoOptions::None,
            game: player_ids(num_players)
                .with_value(|_| "Archipelago".into())
                .collect(),
            completion_condition: player_ids(num_players) //
                .with_value(|_| (|_| true) as _)
                .collect(),
            is_race: false,
            collection_state: CollectionState::new(),
        }
    }

    fn id_is_player(&self, id: SlotId) -> bool {
        1 <= id.0 && id.0 <= self.num_players as i64
    }

    pub fn all_ids(&self) -> impl Iterator<Item = SlotId> {
        self.player_ids().chain(self.groups.keys().copied())
    }

    pub fn add_group(
        &mut self,
        name: SlotName,
        game: GameName,
        players: Set<SlotId>,
    ) -> (SlotId, &mut Group) {
        let existing_group = self.groups.iter_mut().find(|(_, group)| group.name == name);

        if let Some((&group_id, _)) = existing_group {
            // TODO: remove this workaround line once polonius is stabilized (hopefully Rust 1.102?)
            let group = &mut self.groups[&group_id];

            group.players.extend(players);

            return (group_id, group);
        }

        // TODO: get rid of casting the slot id here
        let group_id = SlotId((self.num_players + self.groups.len() + 1) as i64);

        self.region_manager.add_group(group_id);
        self.game.insert(group_id, game.clone());
        self.player_types.insert(group_id, SlotType::Group);

        // TODO: handle 'world not found'
        let world_factory = WORLD_FACTORIES.get(&game).expect("world not found");
        let world = world_factory.new_group(group_id, &players);

        self.worlds.insert(group_id, world);
        // TODO: ensure that `collect_item`, `collect`, and `remove` have fixed behaviour for groups

        self.player_names.insert(group_id, name.clone());

        let group = self
            .groups
            .entry(group_id)
            .insert_entry(Group::new(name, game, players))
            .into_mut();

        (group_id, group)
    }

    pub fn player_groups(&self, player: SlotId) -> impl Iterator<Item = SlotId> {
        self.groups
            .iter()
            .filter_value(move |group| group.players.contains(&player))
            .map_to_key()
            .copied()
    }

    pub fn set_seed(&mut self, seed: Option<Seed>, rng_mode: RngMode, seed_name: Option<String>) {
        // TODO: don't allow seed to change after `worlds` has been initialized (i.e. non-empty)
        self.seed = seed.unwrap_or_else(Seed::new_random);

        match rng_mode {
            RngMode::Seeded => {
                self.random = Random::seeded(self.seed);
            }
            RngMode::Secure => {
                self.random = Random::secure();
                self.is_race = true;
            }
        }

        self.seed_name = seed_name.unwrap_or_else(|| self.seed.to_string());
    }

    // TODO: pub fn set_options(&mut self, args: Namespace)
    // TODO: pub fn set_item_links(&mut self)
    // TODO: pub fn link_items(&mut self)

    // TODO: cache?
    pub fn player_ids(&self) -> impl Iterator<Item = SlotId> {
        player_ids(self.num_players)
    }

    // TODO: cache?
    pub fn game_players(&self, game: &GameName) -> impl Iterator<Item = SlotId> {
        self.player_ids()
            .filter(move |player| &self.game[player] == game)
    }

    // TODO: cache?
    pub fn game_groups(&self, game: &GameName) -> impl Iterator<Item = SlotId> {
        self.groups
            .keys()
            .filter(move |group| &self.game[*group] == game)
            .copied()
    }

    // TODO: cache?
    pub fn game_worlds(&self, game: &GameName) -> impl Iterator<Item = &Arc<dyn World>> {
        self.worlds
            .iter()
            .filter_key(move |&&slot| self.id_is_player(slot) && &self.game[&slot] == game)
            .map_to_value()
    }

    pub fn name_string_for_object<'a, T: HasNameAndPlayer>(&self, object: &'a T) -> Cow<'a, str> {
        if self.num_players == 1 {
            object.name().into()
        } else {
            let name = object.name();
            let player_name = self.player_name(object.player());

            format!("{name} ({player_name})").into()
        }
    }

    pub fn player_name(&self, player: SlotId) -> &SlotName {
        &self.player_names[&player]
    }

    pub fn file_safe_player_name(&self, player: SlotId) -> String {
        utils::file_safe_name(self.player_name(player))
    }

    fn get_out_file_name_base(&self, player: SlotId) -> String {
        let seed_name = &self.seed_name;
        let player_name = self.file_safe_player_name(player).replace(" ", "_");

        format!("AP_{seed_name}_P{player}_{player_name}")
    }

    // TODO: cache?
    pub fn world_name_lookup(&self) -> Map<SlotName, SlotId> {
        self.player_ids()
            .map(|player| (self.player_name(player).clone(), player))
            .collect()
    }

    pub fn regions(&self, player: Option<SlotId>) -> impl Iterator<Item = &Region> {
        let Some(player) = player else {
            return Left(self.region_manager.all_regions());
        };

        Right(
            self.region_manager
                .regions(player)
                .into_iter()
                .flat_map(|regions| regions.values()),
        )
    }

    pub fn region(&self, region_name: &RegionName, player: SlotId) -> Option<&Region> {
        self.region_manager
            .regions(player)
            .and_then(|regions| regions.get(region_name))
    }

    pub fn entrance(&self, entrance_name: &EntranceName, player: SlotId) -> Option<&Entrance> {
        self.region_manager
            .entrances(player)
            .and_then(|regions| regions.get(entrance_name))
    }

    pub fn location(&self, location_name: &LocationName, player: SlotId) -> Option<&Location> {
        self.region_manager
            .locations(player)
            .and_then(|regions| regions.get(location_name))
    }

    // TODO: pub fn all_state(&self)

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.filled_locations(None)
            .filter_map(|location| location.item.as_ref())
            .chain(self.item_pool.iter())
    }

    pub fn find_item_locations(
        &self,
        item_name: &ItemName,
        player: SlotId,
        resolve_group_locations: bool,
    ) -> impl Iterator<Item = &Location> {
        if resolve_group_locations {
            let player_groups = self.player_groups(player).collect::<Set<_>>();

            return Left(self.locations(None).filter(move |location| {
                let Some(item) = &location.item else {
                    return false;
                };

                &item.name == item_name
                    && !player_groups.contains(&location.player)
                    && (item.player == player || player_groups.contains(&item.player))
            }));
        }

        Right(self.locations(None).filter(move |location| {
            let Some(item) = &location.item else {
                return false;
            };

            &item.name == item_name && item.player == player
        }))
    }

    pub fn locations(&self, player: Option<SlotId>) -> impl Iterator<Item = &Location> {
        let Some(player) = player else {
            return Left(self.region_manager.all_locations());
        };

        Right(
            self.region_manager
                .locations(player)
                .into_iter()
                .flat_map(|locations| locations.values()),
        )
    }

    pub fn unfilled_locations(&self, player: Option<SlotId>) -> impl Iterator<Item = &Location> {
        self.locations(player)
            .filter(|location| location.item.is_none())
    }

    pub fn filled_locations(&self, player: Option<SlotId>) -> impl Iterator<Item = &Location> {
        self.locations(player)
            .filter(|location| location.item.is_some())
    }

    // TODO:
    // pub fn reachable_locations(
    //     &self,
    //     collection_state: Option<&CollectionState>,
    //     player: Option<SlotId>,
    // ) {
    //     let collecton_state = collection_state.unwrap_or(&self.collection_state);

    //     self.locations(player)
    //         .filter(|location| location.can_reach(collection_state));
    // }

    // TODO: fn placeable_locations
    // TODO: fn unfilled_locations_for_players
}

trait HasNameAndPlayer {
    fn name(&self) -> &str;
    fn player(&self) -> SlotId;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RngMode {
    Seeded,
    Secure,
}

impl RngMode {
    pub fn is_seeded(&self) -> bool {
        matches!(self, Self::Seeded)
    }

    pub fn is_secure(&self) -> bool {
        matches!(self, Self::Secure)
    }
}

enum Algorithm {
    Balanced,
    Flood,
}

// TODO: figure out what is passed to `completion_condition`
type State = ();

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct PlandoOptions(u32);

bitflags! {
    impl PlandoOptions: u32 {
        const None = 0b0000;
        const Items = 0b0001;
        const Connections = 0b0010;
        const Texts = 0b0100;
        const Bosses = 0b1000;
    }
}
