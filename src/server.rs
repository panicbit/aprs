use std::sync::Arc;
use std::time::Instant;

use eyre::Result;
use fnv::FnvHashMap;
use itertools::Itertools;
use kameo::Actor;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use crate::game::{MultiData, SlotId, TeamId};
use crate::pickle::Value;
use crate::proto::server::{Message, NetworkPlayer};
use crate::server::state::State;

mod event_handlers;
mod state;

mod client;
pub use client::{Client, ClientId};

mod config;
pub use config::Config;

mod event;
pub use event::Event;

#[derive(Actor)]
pub struct Server {
    config: Config,
    multi_data: MultiData,
    // TODO: remove lock after moving to proper client ids
    clients: FnvHashMap<ClientId, Arc<Mutex<Client>>>,
    state: State,
}

impl Server {
    pub fn new(config: Config, multi_data: MultiData) -> Result<Self> {
        let state = match State::try_load(&config.state_path)? {
            Some(state) => {
                info!("Loaded existing state from {:?}", config.state_path);
                state
            }
            None => {
                info!("No existing state found at {:?}", config.state_path);
                State::new(&multi_data)
            }
        };

        Ok(Self {
            config,
            clients: FnvHashMap::default(),
            multi_data,
            state,
        })
    }

    fn get_key(&self, key: &str) -> Option<Value> {
        if let Some(key) = key.strip_prefix("_read_") {
            return self.get_special_key(key);
        }

        self.state.data_storage_get(key)
    }

    fn get_special_key(&self, key: &str) -> Option<Value> {
        if let Some(key) = key.strip_prefix("hints_") {
            return self.get_hints(key);
        }

        if let Some(key) = key.strip_prefix("slot_data_") {
            return self.get_slot_data(key);
        }

        if let Some(key) = key.strip_prefix("item_name_groups_") {
            return self.get_item_name_groups(key);
        }

        if let Some(key) = key.strip_prefix("location_name_groups_") {
            return self.get_location_name_groups(key);
        }

        if let Some(key) = key.strip_prefix("client_status_") {
            return self.get_client_status(key);
        }

        if key == "race_mode" {
            return self.get_race_mode(key);
        }

        error!("Unknown special key: {key}");

        None
    }

    fn get_item_name_groups(&self, key: &str) -> Option<Value> {
        warn!("TODO: implement get_item_name_groups");
        None
    }

    fn get_location_name_groups(&self, key: &str) -> Option<Value> {
        warn!("TODO: implement get_location_name_groups");
        None
    }

    fn get_client_status(&self, key: &str) -> Option<Value> {
        warn!("TODO: implement get_client_status");
        None
    }

    fn get_race_mode(&self, key: &str) -> Option<Value> {
        warn!("TODO: implement get_race_mode");
        None
    }

    fn get_hints(&self, key: &str) -> Option<Value> {
        let (team, slot) = key.split_once("_")?;
        let team = team.parse::<i64>().map(TeamId).ok()?;
        let slot = slot.parse::<i64>().map(SlotId).ok()?;

        let hints = self.state.get_hints(team, slot)?;

        Some(hints)
    }

    fn get_slot_data(&self, key: &str) -> Option<Value> {
        let slot = key.parse::<i64>().map(SlotId).ok()?;

        self.multi_data.slot_data.get(&slot).cloned()
    }

    fn network_players(&self) -> Vec<NetworkPlayer> {
        self.multi_data
            .slot_info
            .iter()
            .map(|(slot_id, slot_info)| NetworkPlayer {
                team: TeamId(0),
                slot: *slot_id,
                // TODO: get alias from state/slot_state
                alias: slot_info.name.as_str().into(),
                name: slot_info.name.clone(),
            })
            .collect_vec()
    }

    async fn broadcast(&self, message: impl Into<Arc<Message>>) {
        let message = message.into();

        for client in self.clients.values() {
            client.lock().await.send(message.clone()).await;
        }
    }

    async fn broadcast_messages(&self, messages: &[Arc<Message>]) {
        for client in self.clients.values() {
            let client = client.lock().await;

            // TODO: allow multiple messages to be sent as a single batch
            for message in messages {
                client.send(message.clone()).await;
            }
        }
    }

    async fn broadcast_slot(&self, slot: SlotId, message: impl Into<Arc<Message>>) {
        let message = message.into();

        for client in self.clients.values() {
            let client = client.lock().await;

            if client.slot_id != slot {
                continue;
            }

            client.send(message.clone()).await;
        }
    }

    async fn sync_items_to_clients(&self) {
        for client in self.clients.values() {
            self.sync_items_to_client(client).await;
        }
    }

    async fn sync_items_to_client(&self, client: &Mutex<Client>) {
        let slot = client.lock().await.slot_id;
        let Some(slot_state) = self.state.get_slot_state(slot) else {
            error!("BUG: trying to sync items to invalid slot {:?}", slot);
            return;
        };
        let slot_received_items = slot_state.received_items();

        client.lock().await.sync_items(slot_received_items).await
    }

    fn save_state(&self) {
        info!("Saving state...");
        let start = Instant::now();
        let result = self.state.save(&self.config.state_path);
        let elapsed = start.elapsed();

        if let Err(err) = result {
            error!("Failed to save state after {elapsed:?}: {err:?}");
        } else {
            info!("Saved state successfuly after {elapsed:?}");
        }
    }
}

impl kameo::prelude::Message<Event> for Server {
    type Reply = ();

    async fn handle(
        &mut self,
        event: Event,
        _ctx: &mut kameo::prelude::Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.on_event(event).await;
    }
}
