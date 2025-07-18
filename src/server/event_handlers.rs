use std::borrow::Cow;
use std::sync::Arc;

use eyre::{ContextCompat, Result, bail};
use fnv::{FnvHashMap, FnvHashSet};
use itertools::Itertools;
use levenshtein::levenshtein;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::game::{LocationId, TeamAndSlot};
use crate::pickle::Value;
use crate::proto::client::{
    Bounce, ClientStatus, Connect, Get, GetDataPackage, LocationChecks, LocationScouts,
    Message as ClientMessage, Messages as ClientMessages, Say, Set, SetNotify, SetOperation,
    StatusUpdate,
};
use crate::proto::common::{Close, Control, Pong};
use crate::proto::server::{
    Bounced, CommandPermission, Connected, ConnectionRefused, DataPackage, DataPackageData,
    LocationInfo, Message, NetworkItem, Permissions, PrintJson, RemainingCommandPermission,
    Retrieved, RoomInfo, RoomUpdate, SetReply, Time,
};
use crate::server::client::{Client, ClientId};
use crate::server::event::Event;

impl super::Server {
    pub async fn on_event(&mut self, event: Event) {
        match event {
            Event::ClientAccepted(client) => self.on_client_accepted(client).await,
            Event::ClientDisconnected(client_id) => {
                self.on_client_disconnected(client_id).await //
            }
            Event::ClientMessages(client_id, messages) => {
                self.on_client_messages(client_id, messages).await
            }
            Event::ClientControl(client_id, control) => {
                self.on_client_control(client_id, control).await
            }
        }
    }

    pub async fn on_client_accepted(&mut self, client: Client) {
        debug!("New client connected: {:?}", client.id());

        // TODO: Generate and assign client id.
        // Stop using ClientId as identifier.

        let client = Arc::new(Mutex::new(client));

        self.clients
            .insert(client.lock().await.id(), client.clone());

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
            })
            .await;
    }

    pub async fn on_client_disconnected(&mut self, client_id: ClientId) {
        info!("Client disconnected: {:?}", client_id);
    }

    pub async fn on_client_messages(&mut self, client_id: ClientId, messages: ClientMessages) {
        let Some(client) = self.clients.get(&client_id).cloned() else {
            return;
        };

        for message in messages {
            if let Err(err) = self.on_client_message(&client, message).await {
                debug!("||| {err:?}");
                client.lock().await.send_control(Close).await;
                self.clients.remove(&client_id);
            }
        }
    }

    pub async fn on_client_control(&mut self, client_id: ClientId, control: Control) {
        let Some(client) = self.clients.get(&client_id).cloned() else {
            return;
        };

        match control {
            Control::Ping(ping) => {
                client.lock().await.send_control(Pong(ping.0)).await;
            }
            Control::Pong(_) => {}
            Control::Close(_) => {
                self.on_close(&client).await;
            }
        }
    }

    pub async fn on_client_message(
        &mut self,
        client: &Mutex<Client>,
        message: ClientMessage,
    ) -> Result<()> {
        // GetDataPackage is allowed to be sent before authenticating
        if let ClientMessage::GetDataPackage(ref get_data_package) = message {
            self.on_get_data_package(client, get_data_package).await;
            return Ok(());
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
            ClientMessage::LocationChecks(location_checks) => {
                self.on_location_checks(client, location_checks).await
            }
            ClientMessage::StatusUpdate(status_update) => {
                self.on_status_update(client, status_update).await
            }
            ClientMessage::Sync(_) => self.on_sync(client).await,
            ClientMessage::GetDataPackage(_) => {
                error!("BUG: GetDataPackage should already be handled as unauthenticated packet")
            }
            ClientMessage::Bounce(bounce) => self.on_bounce(client, &bounce).await,
            ClientMessage::Unknown(value) => warn!("Unknown client message: {value:?}"),
        }

        Ok(())
    }

    pub async fn on_connect(&self, client: &Mutex<Client>, connect: Connect) -> Result<()> {
        let Connect {
            password,
            game,
            name: connect_name,
            uuid,
            version,
            items_handling,
            tags,
            slot_data,
        } = connect;
        // TODO: implement checks:
        // - items handling
        // - version (skip if tags are appropriate)

        // the password must match if one is required
        if let Some(client_password) = &self.multi_data.server_options.client_password {
            if Some(client_password) != password.as_ref() {
                client
                    .lock()
                    .await
                    .send(ConnectionRefused::invalid_password())
                    .await;
                return Ok(());
            }
        }

        // the requested slot name must exist
        let Some(team_and_slot) = self.multi_data.connect_names.get(&connect_name) else {
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return Ok(());
        };
        let TeamAndSlot { slot, team } = *team_and_slot;
        let Some(slot_info) = self.multi_data.slot_info.get(&slot) else {
            error!("Inconsistent multi data!");

            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_slot())
                .await;
            return Ok(());
        };

        let skip_game_and_version_validation = tags
            .iter()
            .any(|tag| ["Tracker", "TextOnly", "HintGame"].contains(&tag.as_str()));

        // the requested slot game must match
        if slot_info.game != game && !skip_game_and_version_validation {
            client
                .lock()
                .await
                .send(ConnectionRefused::invalid_game())
                .await;
            return Ok(());
        }

        {
            let mut client = client.lock().await;

            let slot_state = self
                .state
                .get_slot_state(slot)
                .context("BUG: missing slot state for slot {slot}")?;

            if items_handling.is_starting_inventory() {
                let starting_inventory = self
                    .multi_data
                    .precollected_items
                    .get(&slot)
                    .map(Cow::Borrowed)
                    .unwrap_or_default();

                client.set_starting_inventory(&starting_inventory);
            }

            client.set_items_handling(items_handling);

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
                        .filter(|_| slot_data)
                        .cloned()
                        .unwrap_or_default(),
                    slot_info: self.multi_data.slot_info.clone(),
                    // TODO: sent actual hintpoints
                    hint_points: 0,
                })
                .await;

            client.connect_name = connect_name;
            // TODO: maybe store entire slot_info in client to make slot_info access easier,
            // or separate client into authenticated and unauthenticated types
            client.slot_name = slot_info.name.clone();
            client.slot_id = slot;
            client.team_id = team;
            client.tags = FnvHashSet::from_iter(tags);
            client.game = game;
            client.is_connected = true;
        }

        self.sync_items_to_client(client).await;

        Ok(())
    }

    pub async fn on_say(&mut self, client: &Mutex<Client>, say: Say) {
        let Say { text } = say;
        let text = text.trim();

        let slot = client.lock().await.slot_id;
        let message = PrintJson::builder()
            .with_player(slot)
            .with_text(": ")
            .with_text(text)
            .build();

        self.broadcast(message).await;

        if let Some(item) = text.strip_prefix("!hint ") {
            self.on_command_hint(client, item).await;
        } else if text == "!release" {
            self.on_goal_complete(client).await;
        }
    }

    async fn on_command_hint(&self, client: &Mutex<Client>, needle_item: &str) {
        let needle_item = needle_item.trim();
        let slot = client.lock().await.slot_id;

        if needle_item.is_empty() {
            self.broadcast(PrintJson::chat_message("Usage: !hint <item name>"))
                .await;
            return;
        }

        let Some(slot_info) = self.multi_data.get_slot_info(slot) else {
            error!("BUG: tried to get slot_info for invalid slot {slot:?}");
            return;
        };
        let game = &slot_info.game;
        let Some(game_data) = self.multi_data.get_game_data(game) else {
            error!("BUG: tried to get game data for invalid game {game:?}");
            return;
        };

        let Some((found_item_name, found_item_id, confidence)) = game_data
            .item_name_to_id
            .iter()
            .map(|(item_name, item_id)| {
                let distance = levenshtein(needle_item, item_name);
                let confidence =
                    (distance.min(needle_item.len()) as f32) / (needle_item.len() as f32);
                let confidence = 1.0 - confidence;
                let confidence = (confidence * 100.).floor() as u32;

                (item_name, item_id, confidence)
            })
            .max_by_key(|(_, _, confidence)| *confidence)
        else {
            // TODO: maybe broadcast message in this case
            error!("BUG: world doesn't seem to have any items?");
            return;
        };

        let confidence_threshold = 70;

        if confidence < confidence_threshold {
            self.broadcast(PrintJson::chat_message(format!(
                "No matching item found. Did you mean '{found_item_name}'? ({confidence}% match)"
            )))
            .await;
            return;
        }

        // TOOD: get first uncollected item by sphere
        let Some((item_slot, item_location, flags)) =
            self.multi_data.find_item_location(*found_item_id)
        else {
            // TODO: maybe broadcast message in this case
            error!("BUG: item does not seem to be placed?");
            return;
        };

        self.broadcast(
            PrintJson::builder()
                .with_player(slot)
                .with_text("'s ")
                .with_item(slot, *found_item_id, flags)
                .with_text(" is at ")
                .with_player(item_slot)
                .with_text("'s ")
                .with_location(item_slot, item_location)
                .build(),
        )
        .await;
    }

    pub async fn on_get(&mut self, client: &Mutex<Client>, get: Get) {
        let Get { keys } = get;

        let mut retrieved = FnvHashMap::default();

        for key in keys {
            if let Some(value) = self.get_key(&key) {
                retrieved.insert(key, value);
            }
        }

        client
            .lock()
            .await
            .send(Retrieved { keys: retrieved })
            .await;
    }

    async fn on_set(&mut self, client: &Mutex<Client>, set: Set) {
        let Set {
            key,
            default,
            want_reply,
            operations,
        } = set;

        let slot = client.lock().await.slot_id;
        let original_value = self.state.data_storage_get(&key).unwrap_or(default);
        let mut value = original_value.clone();

        fn handle_op(current: Value, operation: SetOperation) -> Result<Value> {
            Ok(match operation {
                SetOperation::Default => current,
                SetOperation::Replace(value) => value,
                // TODO: implement remaining set ops
                SetOperation::Add(value) => current.add(&value)?,
                SetOperation::Mul(value) => current.mul(&value)?,
                // SetOperation::Pow(value) => current.pow(value),
                // SetOperation::Mod(value) => current.r#mod(value),
                SetOperation::Floor => current.floor()?,
                SetOperation::Ceil => current.ceil()?,
                // SetOperation::Max(value) => current.max(value),
                // SetOperation::Min(value) => current.min(value),
                SetOperation::And(value) => current.and(&value)?,
                SetOperation::Or(value) => current.or(&value)?,
                // SetOperation::Xor(value) => current.xor(value),
                // SetOperation::LeftShift(value) => current.left_shift(value),
                // SetOperation::RightShift(value) => current.right_shift(value),
                // SetOperation::Remove(value) => current.remove(value),
                SetOperation::Pop(value) => current.pop(&value).map(|_| current)?,
                SetOperation::Update(value) => current.update(&value).map(|_| current)?,
                _ => bail!("TODO: implement SetOperation: {operation:?}"),
            })
        }

        self.state.data_storage_set(key.clone(), value.clone());

        for operation in operations {
            value = match handle_op(value, operation) {
                Ok(value) => value,
                Err(err) => {
                    error!("op err: {err:?}");
                    return;
                }
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
        let LocationScouts {
            locations,
            create_as_hint,
        } = location_scouts;
        let slot = client.lock().await.slot_id;

        // TODO: handle create_as_hint

        let locations = locations.into_iter()
            .filter_map(|location_id| {
                let location_info = self.multi_data.location_info(slot, location_id);

                if location_info.is_none() {
                    error!("Client for slot {slot:?} asked for location that does not exist: {location_id:?}")
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

        client.lock().await.send(LocationInfo { locations }).await;
    }

    pub async fn on_location_checks(
        &mut self,
        client: &Mutex<Client>,
        location_checks: LocationChecks,
    ) {
        let LocationChecks { locations } = location_checks;

        self.check_locations(client, locations).await;
    }

    async fn check_locations(&mut self, client: &Mutex<Client>, locations: FnvHashSet<LocationId>) {
        let slot_sending = client.lock().await.slot_id;

        let Some(location_infos) = self.multi_data.get_locations(slot_sending) else {
            error!("BUG: missing location info for slot {slot_sending:?}");
            return;
        };

        let Some(state) = self.state.get_slot_state_mut(slot_sending) else {
            error!("BUG: missing state for slot {slot_sending:?}");
            return;
        };

        let items_by_slot = locations
            .iter()
            .filter_map(|location| {
                if state.check_location(*location).location_was_checked() {
                    return None;
                }

                let location_info = location_infos.get(location)?;
                let network_item = NetworkItem {
                    item: location_info.item,
                    location: *location,
                    player: slot_sending,
                    flags: location_info.flags,
                };

                Some((location_info.slot, network_item))
            })
            .into_group_map();

        // we have nothing to do
        if items_by_slot.is_empty() {
            return;
        }

        let mut chat_messages = Vec::new();

        for (slot_receiving, items) in items_by_slot {
            let Some(slot_state) = self.state.get_slot_state_mut(slot_receiving) else {
                error!("Tried to add items to invalid slot {slot_receiving:?}");
                continue;
            };

            for item in &items {
                let message = PrintJson::chat_message_for_received_item(*item, slot_receiving);

                chat_messages.push(message.into());
            }

            slot_state.add_received_items(items);
        }

        self.save_state();
        self.broadcast_slot(slot_sending, RoomUpdate::checked_locations(locations))
            .await;
        self.sync_items_to_clients().await;
        self.broadcast_messages(&chat_messages).await;
        // TODO: send RoomUpdate for checked_locations
    }

    pub async fn on_status_update(&mut self, client: &Mutex<Client>, status_update: StatusUpdate) {
        let StatusUpdate { status } = status_update;

        // TODO: handle other status updates
        match status {
            ClientStatus::Unknown => {}
            ClientStatus::Connected => {}
            ClientStatus::Ready => {}
            ClientStatus::Playing => {}
            ClientStatus::Goal => {
                self.on_goal_complete(client).await;
            }
        };
    }

    pub async fn on_goal_complete(&mut self, client: &Mutex<Client>) {
        let slot = client.lock().await.slot_id;
        let Some(slot_state) = self.state.get_slot_state(slot) else {
            error!("Tried to get slot state for unknown slot {slot:?}");
            return;
        };
        let missing_locations = slot_state.missing_locations().clone();

        // TODO: handle disabled autocollect
        self.check_locations(client, missing_locations).await;
    }

    pub async fn on_get_data_package(
        &mut self,
        client: &Mutex<Client>,
        get_data_package: &GetDataPackage,
    ) {
        let GetDataPackage { games } = get_data_package;

        let client = client.lock().await;

        // TODO: only send back games asked for
        client
            .send(DataPackage {
                data: DataPackageData {
                    games: self.multi_data.data_package.clone(),
                },
            })
            .await;
    }

    pub async fn on_sync(&mut self, client: &Mutex<Client>) {
        client.lock().await.reset_received_items();
        self.sync_items_to_client(client).await;
    }

    pub async fn on_bounce(&mut self, client: &Mutex<Client>, bounce: &Bounce) {
        let bounced = Bounced::from(bounce.clone());
        let bounced = Arc::<Message>::from(bounced);
        let Bounce {
            games,
            slots,
            tags,
            data: _,
        } = bounce;

        let team_id = client.lock().await.team_id;

        for client in self.clients.values() {
            let client = client.lock().await;

            let team_matches = || client.team_id != team_id;
            let game_matches = || games.contains(&client.game);
            let team_and_game_matches = || team_matches() && game_matches();
            let tag_matches = || tags.iter().any(|tag| client.tags.contains(tag));
            let slot_matches = || slots.contains(&client.slot_id);

            if team_and_game_matches() || tag_matches() || slot_matches() {
                client.send(bounced.clone()).await;
            }
        }
    }

    pub async fn on_close(&mut self, client: &Mutex<Client>) {
        self.clients.remove(&client.lock().await.id());
    }
}
