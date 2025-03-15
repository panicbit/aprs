use fnv::{FnvHashMap, FnvHashSet};

use crate::game::{LocationId, MultiData, SlotId, TeamId};
use crate::pickle::Value;
use crate::pickle::value::Str;

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

    pub fn data_storage_get(&self, key: &str) -> Option<Value> {
        self.data_storage.get(key).cloned()
    }

    pub fn data_storage_set(&mut self, key: Str, value: impl Into<Value>) {
        self.data_storage.insert(key, value.into());
    }

    pub fn get_hints(&self, team: TeamId, slot: SlotId) -> Option<Value> {
        // TODO: implement get hints
        eprintln!("TODO: implement get_hints");
        None
    }
}

pub struct SlotState {
    missing_locations: FnvHashSet<LocationId>,
    checked_locations: FnvHashSet<LocationId>,
}

impl SlotState {
    pub fn new(multi_data: &MultiData, slot: SlotId) -> Self {
        Self {
            missing_locations: multi_data.location_ids(slot).collect(),
            checked_locations: FnvHashSet::default(),
        }
    }

    pub fn missing_locations(&self) -> &FnvHashSet<LocationId> {
        &self.missing_locations
    }

    pub fn checked_locations(&self) -> &FnvHashSet<LocationId> {
        &self.checked_locations
    }
}
