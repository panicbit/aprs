use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use fnv::FnvHashMap;
use serde_json::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;

use crate::game::{MultiData, SlotId, TeamId};
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
    pub fn new(config: Config, multi_data: MultiData) -> Self {
        let (tx, rx) = mpsc::channel(10_000);

        Self {
            config,
            tx,
            rx,
            clients: FnvHashMap::default(),
            state: State::new(&multi_data),
            multi_data,
        }
    }

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
                eprintln!("Event channel closed.");
                return;
            };

            self.on_event(event).await;
        }
    }

    fn get_key(&self, key: &str) -> Option<Arc<Value>> {
        if let Some(key) = key.strip_prefix("_read_") {
            return self.get_special_key(key);
        }

        self.state.data_storage_get(key).map(Arc::new)
    }

    fn get_special_key(&self, key: &str) -> Option<Arc<Value>> {
        if let Some(key) = key.strip_prefix("hints_") {
            return self.get_hints(key);
        }

        if let Some(key) = key.strip_prefix("slot_data_") {
            return self.get_slot_data(key);
        }

        todo!()
    }

    fn get_hints(&self, key: &str) -> Option<Arc<Value>> {
        let (team, slot) = key.split_once("_")?;
        let team = team.parse::<i64>().map(TeamId).ok()?;
        let slot = slot.parse::<i64>().map(SlotId).ok()?;

        let hints = self.state.get_hints(team, slot)?;

        Some(Arc::new(hints))
    }

    fn get_slot_data(&self, key: &str) -> Option<Arc<Value>> {
        let slot = key.parse::<i64>().map(SlotId).ok()?;

        self.multi_data.slot_data.get(&slot).cloned()
    }

    fn network_players(&self) -> Vec<NetworkPlayer> {
        self.multi_data
            .connect_names
            .iter()
            .map(|(slot_name, ts)| NetworkPlayer {
                team: ts.team,
                slot: ts.slot,
                // TODO: get alias from state/slot_state
                alias: slot_name.as_str().to_owned(),
                name: slot_name.clone(),
            })
            .collect()
    }

    async fn broadcast(&self, message: impl Into<Arc<Message>>) {
        let message = message.into();

        for client in self.clients.values() {
            client.lock().await.send(message.clone()).await;
        }
    }
}

async fn acceptor_loop(listener: TcpListener, event_tx: Sender<Event>) {
    loop {
        select! {
            _ = event_tx.closed() => {
                eprintln!("acceptor loop shutting down");
                return
            },
            accepted = listener.accept() => {
                let (stream, address) = match accepted {
                    Ok(client) => client,
                    Err(err) => {
                        eprintln!("Error accepting client: {err:?}");
                        continue;
                    }
                };

                let event_tx = event_tx.clone();

                tokio::spawn(async move {
                    if let Err(err) = handle_accept(stream, address, event_tx).await {
                        eprintln!("Failed to accept client {address}: {err:?}");
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
    eprintln!("||| {address:?} connected");

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
        eprintln!("Can't accept client, event channel is closed");
    }

    Ok(())
}
