use std::net::SocketAddr;
use std::sync::Arc;

use archipelago_core::game::TeamAndSlot;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;

use crate::proto::client::{Connect, Message as ClientMessage, Messages as ClientMessages, Say};
use crate::proto::common::{Ping, Pong};
use crate::proto::server::{
    CommandPermission, Connected, ConnectionRefused, Permissions, PrintJson,
    RemainingCommandPermission, RoomInfo, Time,
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
                generator_version: self.multi_data.version.into(),
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
            // TODO: bail early on error
            self.on_client_message(&client, message).await;
        }
    }

    pub async fn on_client_message(&mut self, client: &Mutex<Client>, message: ClientMessage) {
        match message {
            ClientMessage::Ping(ping) => {
                client.lock().await.send(Pong(ping.0)).await;
                return;
            }
            ClientMessage::Pong(_) => {
                return;
            }
            ClientMessage::Close(_) => {
                self.on_close(client).await;
                return;
            }
            _ => {}
        }

        if !client.lock().await.is_connected {
            match message {
                ClientMessage::Connect(connect) => self.on_connect(client, connect).await,
                _ => {
                    eprintln!("Client sent non-connect message before being connected: {message:?}")
                }
            }

            return;
        }

        match message {
            ClientMessage::Say(say) => self.on_say(client, say).await,
            ClientMessage::Connect(_) => {
                eprintln!("Client already connected, but send another connect mesage; ignoring")
            }
            _ => eprintln!("{}: {message:#?}", client.lock().await.address),
        }
    }

    pub async fn on_connect(&self, client: &Mutex<Client>, connect: Connect) {
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
                return;
            }
        }

        // the requested slot name must exist
        let Some(team_and_slot) = self.multi_data.connect_names.get(&connect.name) else {
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return;
        };
        let TeamAndSlot { slot, team } = *team_and_slot;
        let Some(slot_info) = self.multi_data.slot_info.get(&slot) else {
            eprintln!("Inconsistent multi data!");
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return;
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
            return;
        }

        let mut client = client.lock().await;

        client
            .send(Connected {
                team,
                slot,
                // TODO: send list of NetworkPlayers
                players: vec![],
                // TODO: send list of missing locations
                missing_locations: vec![],
                // TODO: send list of checked locations
                checked_locations: vec![],
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
        client.is_connected = true;
    }

    pub async fn on_say(&mut self, client: &Mutex<Client>, say: Say) {
        let name = client.lock().await.slot_name.clone();

        for client in self.clients.values() {
            client
                .lock()
                .await
                .send(PrintJson::chat_message(format!("{name}: {}", say.text)))
                .await;
        }
    }

    pub async fn on_close(&mut self, client: &Mutex<Client>) {
        self.clients.remove(&client.lock().await.address);
    }
}
