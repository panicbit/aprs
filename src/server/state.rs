use std::borrow::Cow;
use std::fs::File;
use std::io;
use std::path::Path;

use aprs_proto::primitives::{LocationId, SlotId, TeamId};
use aprs_proto::server::NetworkItem;
use aprs_server_core::DataStorage;
use aprs_value::{Str, Value};
use color_eyre::eyre::{ContextCompat, Result};
use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use tempfile::NamedTempFile;
use tracing::warn;

use crate::game::MultiData;

#[derive(Deserialize, Serialize)]
pub struct State {
    slot_states: FnvHashMap<SlotId, SlotState>,
    pub data_storage: DataStorage,
}

impl State {
    pub fn new(multi_data: &MultiData) -> Self {
        let slot_states = multi_data
            .slot_ids()
            .map(|slot| (slot, SlotState::new(multi_data, slot)))
            .collect::<FnvHashMap<_, _>>();

        Self {
            slot_states,
            data_storage: DataStorage::new(),
        }
    }

    pub fn try_load(path: &Path) -> Result<Option<Self>> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let decoder = zstd::Decoder::new(file)?;
        let state = rmp_serde::from_read::<_, State>(decoder)?;

        Ok(Some(state))
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let dir = path.parent().context("invalid save path")?;
        let file = NamedTempFile::new_in(dir)?;
        let mut encoder = zstd::Encoder::new(file, zstd::DEFAULT_COMPRESSION_LEVEL)?;

        rmp_serde::encode::write_named(&mut encoder, self)?;

        let file = encoder.finish()?;
        file.persist(path)?;

        Ok(())
    }

    pub fn get_slot_state(&self, slot: SlotId) -> Option<&SlotState> {
        self.slot_states.get(&slot)
    }

    pub fn get_slot_state_mut(&mut self, slot: SlotId) -> Option<&mut SlotState> {
        self.slot_states.get_mut(&slot)
    }

    pub fn get_hints(&self, _team: TeamId, _slot: SlotId) -> Option<Value> {
        // TODO: implement get hints
        warn!("TODO: implement get_hints");
        None
    }
}

#[derive(Deserialize, Serialize)]
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
                player: SlotId::SERVER,
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
