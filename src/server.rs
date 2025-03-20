use std::iter;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use eyre::Result;
use fnv::FnvHashMap;
use itertools::Itertools;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;
use tracing::{debug, error, info, instrument, warn};

use crate::game::{MultiData, SlotId, SlotName, TeamId};
use crate::pickle::Value;
use crate::proto::server::{Message, NetworkPlayer};
use crate::server::client::Client;
use crate::server::event::Event;
use crate::server::state::State;

pub use config::Config;

mod client;
mod config;
mod event;
mod event_handlers;
mod state;

pub struct Server {
    config: Config,
    multi_data: MultiData,
    rx: Receiver<Event>,
    tx: Sender<Event>,
    // TODO: remove lock after moving to proper client ids
    clients: FnvHashMap<SocketAddr, Arc<Mutex<Client>>>,
    state: State,
}

impl Server {
    pub fn new(config: Config, multi_data: MultiData) -> Result<Self> {
        let (tx, rx) = mpsc::channel(10_000);

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
            tx,
            rx,
            clients: FnvHashMap::default(),
            multi_data,
            state,
        })
    }

    #[instrument(skip_all)]
    pub async fn run(self) -> Result<()> {
        let listen_address = self.config.listen_address;
        let listener = TcpListener::bind(listen_address).await?;

        tokio::spawn(acceptor_loop(listener, self.tx.clone()));

        self.event_loop().await;

        Ok(())
    }

    pub async fn event_loop(mut self) {
        loop {
            let Some(event) = self.rx.recv().await else {
                debug!("Event channel closed.");
                return;
            };

            self.on_event(event).await;
        }
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

        if let Some(key) = key.strip_prefix("client_status_") {
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
        let archipelago = NetworkPlayer {
            team: TeamId(0),
            slot: SlotId(0),
            alias: "Archipelago".into(),
            name: SlotName("Archipelago".into()),
        };

        let players = self
            .multi_data
            .slot_info
            .iter()
            .map(|(slot_id, slot_info)| NetworkPlayer {
                team: TeamId(0),
                slot: *slot_id,
                // TODO: get alias from state/slot_state
                alias: slot_info.name.as_str().into(),
                name: slot_info.name.clone(),
            });

        let players = iter::once(archipelago).chain(players).collect_vec();

        players
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

            for message in messages {
                client.send(message.clone()).await;
            }
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

async fn acceptor_loop(listener: TcpListener, event_tx: Sender<Event>) {
    loop {
        select! {
            _ = event_tx.closed() => {
                debug!("acceptor loop shutting down");
                return
            },
            accepted = listener.accept() => {
                let (stream, address) = match accepted {
                    Ok(client) => client,
                    Err(err) => {
                        error!("Error accepting client: {err:?}");
                        continue;
                    }
                };

                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    if let Err(err) = handle_accept(stream, address, event_tx).await {
                        error!("Failed to accept client {address}: {err:?}");
                    }
                });
            }
        }
    }
}

async fn handle_accept(
    stream: TcpStream,
    address: SocketAddr,
    event_tx: Sender<Event>,
) -> Result<()> {
    debug!("||| {address:?} connected");

    #[derive(Default)]
    struct Data {
        uri: Uri,
    }

    impl Callback for &mut Data {
        fn on_request(
            self,
            request: &tokio_tungstenite::tungstenite::handshake::server::Request,
            response: tokio_tungstenite::tungstenite::handshake::server::Response,
        ) -> std::result::Result<
            tokio_tungstenite::tungstenite::handshake::server::Response,
            tokio_tungstenite::tungstenite::handshake::server::ErrorResponse,
        > {
            self.uri = request.uri().clone();

            Ok(response)
        }
    }

    let mut data = Data::default();
    let stream = tokio_tungstenite::accept_hdr_async(stream, &mut data).await?;

    if event_tx
        .send(Event::ClientAccepted(address, stream))
        .await
        .is_err()
    {
        debug!("Can't accept client, event channel is closed");
    }

    Ok(())
}
