use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use fnv::FnvHashMap;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

use crate::game::TeamAndSlot;
use crate::pickle::Value;
use crate::proto::client::{
    Connect, Get, LocationScouts, Message as ClientMessage, Messages as ClientMessages, Say, Set, SetNotify, SetOperation
};
use crate::proto::common::{Close, Ping, Pong};
use crate::proto::server::{
    CommandPermission, Connected, ConnectionRefused, LocationInfo, Message, NetworkItem, Permissions, PrintJson, RemainingCommandPermission, Retrieved, RoomInfo, SetReply, Time
};
use crate::server::client::Client;
use crate::server::event::Event;

impl super::Server {
    pub async fn on_event(&mut self, event: Event) {
        match event {
            Event::ClientAccepted(address, stream) => {
                self.on_client_accepted(address, stream).await
            }
            Event::ClientDisconnected(address) => {
                self.on_client_disconnected(address).await //
            }
            Event::ClientMessages(address, messages) => {
                self.on_client_messages(address, messages).await
            }
        }
    }

    pub async fn on_client_accepted(
        &mut self,
        address: SocketAddr,
        stream: WebSocketStream<TcpStream>,
    ) {
        eprintln!("New client connected: {}", address);

        // TODO: Generate and assign client id.
        // Stop using SocketAddr as identifier.

        let client = Client::new(address, stream, self.tx.clone());
        let client = Arc::new(Mutex::new(client));

        self.clients.insert(address, client.clone());

        client.lock().await.send(Ping("hello".into())).await;

        client
            .lock()
            .await
            .send(RoomInfo {
                version: (0, 5, 1).into(),
                generator_version: self.multi_data.version,
                tags: vec!["AP".into(), "Rust".into()],
                password: self.multi_data.server_options.client_password.is_some(),
                permissions: Permissions {
                    release: CommandPermission::Auto,
                    collect: CommandPermission::Auto,
                    remaining: RemainingCommandPermission::Enabled,
                },
                // TODO: set hint cost properly
                hint_cost: 10,
                // TODO: set location check points properly
                location_check_points: 20,
                games: self.multi_data.data_package.keys().cloned().collect(),
                datapackage_checksums: self
                    .multi_data
                    .data_package
                    .iter()
                    .map(|(game, dp)| (game.clone(), dp.checksum.clone()))
                    .collect(),
                seed_name: self.multi_data.seed_name.0.clone(),
                time: Time::now(),
                // should be empty
                players: vec![],
            })
            .await;
    }

    pub async fn on_client_disconnected(&mut self, address: SocketAddr) {
        eprintln!("Client connected: {}", address);
    }

    pub async fn on_client_messages(&mut self, address: SocketAddr, messages: ClientMessages) {
        let Some(client) = self.clients.get(&address).cloned() else {
            return;
        };

        for message in messages {
            if let Err(err) = self.on_client_message(&client, message).await {
                eprintln!("||| {err:?}");
                client.lock().await.send(Close).await;
                self.clients.remove(&address);
            }
        }
    }

    pub async fn on_client_message(
        &mut self,
        client: &Mutex<Client>,
        message: ClientMessage,
    ) -> Result<()> {
        match message {
            ClientMessage::Ping(ping) => {
                client.lock().await.send(Pong(ping.0)).await;
                return Ok(());
            }
            ClientMessage::Pong(_) => {
                return Ok(());
            }
            ClientMessage::Close(_) => {
                self.on_close(client).await;
                return Ok(());
            }
            _ => {}
        }

        if !client.lock().await.is_connected {
            match message {
                ClientMessage::Connect(connect) => self.on_connect(client, connect).await?,
                _ => {
                    bail!("Client sent non-connect message before being connected: {message:?}")
                }
            }

            return Ok(());
        }

        match message {
            ClientMessage::Connect(_) => {
                bail!("Client already connected, but sent another connect message")
            }
            ClientMessage::Say(say) => self.on_say(client, say).await,
            ClientMessage::Get(get) => self.on_get(client, get).await,
            ClientMessage::Set(set) => self.on_set(client, set).await,
            ClientMessage::SetNotify(set_notify) => self.on_set_notify(client, set_notify).await,
            ClientMessage::LocationScouts(location_scouts) => {
                self.on_location_scouts(client, location_scouts).await
            }
            ClientMessage::Unknown(value) => eprintln!("Unknown client message: {value:?}"),
            ClientMessage::Ping(_) => bail!("unreachable: Ping"),
            ClientMessage::Pong(_) => bail!("unreachable: Pong"),
            ClientMessage::Close(_) => bail!("unreachable: Close"),
        }

        Ok(())
    }

    pub async fn on_connect(&self, client: &Mutex<Client>, connect: Connect) -> Result<()> {
        // TODO: implement checks:
        // - items handling
        // - version (skip if tags are appropriate)

        // the password must match if one is required
        if let Some(password) = &self.multi_data.server_options.client_password {
            if Some(password) != connect.password.as_ref() {
                client
                    .lock()
                    .await
                    .send(ConnectionRefused::invalid_password())
                    .await;
                return Ok(());
            }
        }

        // the requested slot name must exist
        let Some(team_and_slot) = self.multi_data.connect_names.get(&connect.name) else {
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return Ok(());
        };
        let TeamAndSlot { slot, team } = *team_and_slot;
        let Some(slot_info) = self.multi_data.slot_info.get(&slot) else {
            eprintln!("Inconsistent multi data!");
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return Ok(());
        };

        let skip_game_and_version_validation = connect
            .tags
            .iter()
            .any(|tag| ["Tracker", "TextOnly", "HintGame"].contains(&tag.as_str()));

        // the requested slot game must match
        if slot_info.game != connect.game && !skip_game_and_version_validation {
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_game())
                .await;
            return Ok(());
        }

        let mut client = client.lock().await;

        let slot_state = self
            .state
            .get_slot_state(slot)
            .context("BUG: missing slot state for slot {slot}")?;

        client
            .send(Connected {
                team,
                slot,
                players: self.network_players(),
                missing_locations: slot_state.missing_locations().clone(),
                checked_locations: slot_state.checked_locations().clone(),
                slot_data: self
                    .multi_data
                    .slot_data
                    .get(&slot)
                    .filter(|_| connect.slot_data)
                    .cloned()
                    .unwrap_or_default(),
                slot_info: self.multi_data.slot_info.clone(),
                // TODO: sent actual hintpoints
                hint_points: 0,
            })
            .await;

        client.slot_name = connect.name;
        client.slot_id = slot;
        client.is_connected = true;

        Ok(())
    }

    pub async fn on_say(&mut self, client: &Mutex<Client>, say: Say) {
        let name = client.lock().await.slot_name.clone();
        let message = PrintJson::chat_message(format!("{name}: {}", say.text));

        self.broadcast(message).await;
    }

    pub async fn on_get(&mut self, client: &Mutex<Client>, get: Get) {
        let Get { keys } = get;

        let mut retrieved = FnvHashMap::default();

        for key in keys {
            if let Some(value) = self.get_key(&key) {
                retrieved.insert(key, value);
            }
        }
        
        client.lock().await.send(Retrieved { keys: retrieved }).await;
    }

    async fn on_set(&self, client: &Mutex<Client>, set: Set) {
        let Set { key, default, want_reply, operations } = set;
        
        let slot = client.lock().await.slot_id;
        let original_value = self.state.data_storage_get(&key).unwrap_or(default);
        let mut value = original_value.clone();

        fn handle_op(current: Value, operation: SetOperation) -> Result<Value> {
            Ok(match operation {
                SetOperation::Default => current,
                SetOperation::Replace(value) => value,
                // TODO: implement remaining set ops
                // SetOperation::Add(value) => current.add(value),
                // SetOperation::Mul(value) => current.mul(value),
                // SetOperation::Pow(value) => current.pow(value),
                // SetOperation::Mod(value) => current.r#mod(value),
                // SetOperation::Floor => current.floor(),
                // SetOperation::Ceil => current.ceil(),
                // SetOperation::Max(value) => current.max(value),
                // SetOperation::Min(value) => current.min(value),
                // SetOperation::And(value) => current.and(value),
                // SetOperation::Or(value) => current.or(value),
                // SetOperation::Xor(value) => current.xor(value),
                // SetOperation::LeftShift(value) => current.left_shift(value),
                // SetOperation::RightShift(value) => current.right_shift(value),
                // SetOperation::Remove(value) => current.remove(value),
                // SetOperation::Pop(value) => current.pop(value),
                SetOperation::Update(value) => {
                    {

                        let current = current.as_dict()?;
                        let value = value.as_dict()?;
                        
                        for (key, value) in value {
                            current.insert(key, value)?;
                        }
                    }

                    value
                },
                _ => bail!("TODO: implement SetOperation: {operation:?}"),
            })
        }

        for operation in operations {
            value = match handle_op(value, operation) {
                Ok(value) => value,
                Err(err) => {
                    eprintln!("op err: {err:?}");
                    return;
                },
            }
        }

        let set_reply = Arc::new(Message::SetReply(SetReply {
            key: key.clone(),
            value,
            original_value,
            slot,
        }));

        {
            let client = client.lock().await;

            if want_reply && !client.wants_updates_for_keys.contains(&key) {
                client.send(set_reply.clone()).await;
            }
        }

        for client in self.clients.values() {
            let client = client.lock().await;
            
            if client.wants_updates_for_keys.contains(&key) {
                client.send(set_reply.clone()).await;
            }
        }
    }

    pub async fn on_set_notify(&mut self, client: &Mutex<Client>, set_notify: SetNotify) {
        let SetNotify { keys } = set_notify;

        client.lock().await.wants_updates_for_keys = keys;
    }

    pub async fn on_location_scouts(
        &mut self,
        client: &Mutex<Client>,
        location_scouts: LocationScouts,
    ) {
        // TODO: handle `create_as_hint`
        let LocationScouts { locations, create_as_hint } = location_scouts;
        let slot = client.lock().await.slot_id;

        let locations = locations.into_iter()
            .filter_map(|location_id| {
                let location_info = self.multi_data.location_info(slot, location_id); 

                if location_info.is_none() {
                    eprintln!("Client for slot {slot:?} asked for location that does not exist: {location_id:?}")
                }

                Some((location_id, location_info?))
            })
            .map(|(location_id, location_info)| {
                NetworkItem {
                    item: location_info.item,
                    location: location_id,
                    player: slot,
                    flags: location_info.flags,
                }
            })
            .collect::<Vec<_>>();

        client.lock().await.send(LocationInfo {
            locations,
        }).await;
    }

    pub async fn on_close(&mut self, client: &Mutex<Client>) {
        self.clients.remove(&client.lock().await.address);
    }
}
