use std::borrow::Cow;

use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use tracing::warn;

use crate::game::{LocationId, MultiData, SlotId, TeamId};
use crate::pickle::Value;
use crate::pickle::value::Str;
use crate::proto::server::NetworkItem;

pub struct State {
    slot_states: FnvHashMap<SlotId, SlotState>,
    data_storage: FnvHashMap<Str, Value>,
}

impl State {
    pub fn new(multi_data: &MultiData) -> Self {
        let slot_states = multi_data
            .slot_ids()
            .map(|slot| (slot, SlotState::new(multi_data, slot)))
            .collect::<FnvHashMap<_, _>>();

        Self {
            slot_states,
            data_storage: FnvHashMap::default(),
        }
    }

    pub fn get_slot_state(&self, slot: SlotId) -> Option<&SlotState> {
        self.slot_states.get(&slot)
    }

    pub fn get_slot_state_mut(&mut self, slot: SlotId) -> Option<&mut SlotState> {
        self.slot_states.get_mut(&slot)
    }

    pub fn data_storage_get(&self, key: &str) -> Option<Value> {
        self.data_storage.get(key).cloned()
    }

    pub fn data_storage_set(&mut self, key: Str, value: impl Into<Value>) {
        self.data_storage.insert(key, value.into());
    }

    pub fn get_hints(&self, team: TeamId, slot: SlotId) -> Option<Value> {
        // TODO: implement get hints
        warn!("TODO: implement get_hints");
        None
    }
}

pub struct SlotState {
    missing_locations: FnvHashSet<LocationId>,
    checked_locations: FnvHashSet<LocationId>,
    received_items: Vec<NetworkItem>,
}

impl SlotState {
    pub fn new(multi_data: &MultiData, slot: SlotId) -> Self {
        let starting_inventory = multi_data
            .precollected_items
            .get(&slot)
            .map(Cow::Borrowed)
            .unwrap_or_default()
            .iter()
            .map(|&item| NetworkItem {
                item,
                location: LocationId(0),
                player: slot,
                flags: 0,
            })
            .collect_vec();

        Self {
            missing_locations: multi_data.location_ids(slot).collect(),
            checked_locations: FnvHashSet::default(),
            received_items: starting_inventory,
        }
    }

    pub fn missing_locations(&self) -> &FnvHashSet<LocationId> {
        &self.missing_locations
    }

    pub fn checked_locations(&self) -> &FnvHashSet<LocationId> {
        &self.checked_locations
    }

    pub fn check_location(&mut self, location: LocationId) -> CheckOutcome {
        if !self.missing_locations.remove(&location) {
            return CheckOutcome::LocationWasChecked;
        }

        self.checked_locations.insert(location);

        CheckOutcome::LocationWasUnchecked
    }

    pub fn add_received_items(&mut self, items: impl IntoIterator<Item = NetworkItem>) {
        self.received_items.extend(items);
    }

    pub fn received_items(&self) -> &[NetworkItem] {
        &self.received_items
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckOutcome {
    LocationWasChecked,
    LocationWasUnchecked,
}

impl CheckOutcome {
    pub fn location_was_checked(&self) -> bool {
        matches!(self, CheckOutcome::LocationWasChecked)
    }

    pub fn location_was_unchecked(&self) -> bool {
        matches!(self, CheckOutcome::LocationWasUnchecked)
    }
}
