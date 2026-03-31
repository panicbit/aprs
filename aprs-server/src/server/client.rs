use std::sync::Arc;

use aprs_proto as proto;
use aprs_proto::client::ItemsHandling;
use aprs_proto::primitives::{ConnectName, ItemId, SlotId, SlotName, TeamId};
use aprs_proto::server::NetworkItem;
use aprs_proto::server::ReceivedItems;
use aprs_server_core::traits::{GetGame, GetSlotId, GetTeamId, HasTag};
use aprs_value::Str;
use fnv::FnvHashSet;
use itertools::Itertools;
use tracing::{error, info};

use crate::server::client_id::ClientId;
use crate::server::control::{Close, Control, ControlOrMessage};
use crate::server::{Event, Server, ServerMessage, ServerMessageSender, ServerToClientConnection};

#[derive(Clone)]
pub(super) struct Client {
    client_message_sender: ServerMessageSender,
    pub id: ClientId,
    pub is_connected: bool,
    pub connect_name: ConnectName,
    pub slot_name: SlotName,
    pub slot_id: SlotId,
    pub team_id: TeamId,
    pub tags: FnvHashSet<String>,
    pub game: String,
    pub wants_updates_for_keys: FnvHashSet<Str>,
    starting_inventory: FnvHashSet<ItemId>,
    items_handling: ItemsHandling,
    next_slot_item_index: usize,
    next_client_item_index: usize,
}

impl Client {
    pub fn new(
        id: ClientId,
        server: &Server,
        server_to_client_connection: ServerToClientConnection,
    ) -> Self {
        let (client_message_sender, mut server_message_receiver) =
            server_to_client_connection.split();

        // TODO: somehow avoid having to forward messages from the individual
        // client->server channels into the singular server control channel.
        // At the very least try to avoid having multiple tasks somehow.
        {
            let server_message_sender = server.client_message_sender.clone();

            tokio::spawn(async move {
                while let Some(control_or_message) = server_message_receiver.recv().await {
                    let event = match control_or_message {
                        ControlOrMessage::Control(control) => Event::ClientControl(id, control),
                        ControlOrMessage::Message(messages) => Event::ClientMessages(id, messages),
                    };

                    if server_message_sender.send(event).await.is_err() {
                        break;
                    }
                }

                info!("client->server message forwarder stopping");
            });
        }

        Self {
            id,
            client_message_sender,
            is_connected: false,
            connect_name: ConnectName::new(),
            slot_name: SlotName::new(),
            slot_id: SlotId(-1),
            team_id: TeamId(-1),
            game: "<unknown>".into(),
            tags: FnvHashSet::default(),
            wants_updates_for_keys: FnvHashSet::default(),
            starting_inventory: FnvHashSet::default(),
            items_handling: ItemsHandling::empty(),
            next_slot_item_index: 0,
            next_client_item_index: 0,
        }
    }

    pub async fn send(&self, message: impl Into<Arc<ServerMessage>>) {
        // TODO: handle overload situation, probably using timeout
        self.send_control_or_message(ControlOrMessage::Message(message.into()))
            .await
    }

    pub async fn send_control(&self, control: impl Into<Control>) {
        // TODO: handle overload situation, probably using timeout
        self.send_control_or_message(control.into().into()).await
    }

    pub async fn send_control_or_message(&self, message: ControlOrMessage<Arc<ServerMessage>>) {
        // TODO: handle overload situation, probably using timeout
        self.client_message_sender.send(message).await.ok();
    }

    pub async fn close(&self) {
        self.send_control(Close).await
    }

    pub fn set_items_handling(&mut self, new_items_handling: proto::client::ItemsHandling) {
        if new_items_handling == self.items_handling {
            return;
        }

        self.items_handling = new_items_handling;
        self.reset_received_items();
    }

    pub fn set_starting_inventory(&mut self, starting_inventory: &[ItemId]) {
        self.starting_inventory = starting_inventory
            .iter()
            .copied()
            .collect::<FnvHashSet<_>>();
    }

    pub fn reset_received_items(&mut self) {
        self.next_slot_item_index = 0;
        self.next_client_item_index = 0;
    }

    pub async fn sync_items(&mut self, slot_items: &[NetworkItem]) {
        let Some(missing_items) = slot_items.get(self.next_slot_item_index..) else {
            error!("BUG: next_slot_item_index out of bounds");
            return;
        };

        let client_index = self.next_client_item_index;
        let missing_items = missing_items
            .iter()
            .filter(|item| {
                if !self.items_handling.is_remote() {
                    return false;
                }

                if item.player != self.slot_id {
                    return true;
                }

                if self.items_handling.is_starting_inventory()
                    && item.player.is_server()
                    && self.starting_inventory.contains(&item.item)
                {
                    return true;
                }

                self.items_handling.is_own_world()
            })
            .copied()
            .collect_vec();

        self.next_client_item_index += missing_items.len();
        self.next_slot_item_index = slot_items.len();

        if missing_items.is_empty() {
            return;
        }

        self.send(ReceivedItems {
            index: client_index,
            items: missing_items,
        })
        .await;
    }
}

impl GetSlotId for Client {
    fn get_slot_id(&self) -> SlotId {
        self.slot_id
    }
}

impl GetTeamId for Client {
    fn get_team_id(&self) -> TeamId {
        self.team_id
    }
}

impl GetGame for Client {
    fn get_game(&self) -> &str {
        &self.game
    }
}

impl HasTag for Client {
    fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}
