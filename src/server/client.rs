use std::net::SocketAddr;
use std::sync::Arc;

use fnv::FnvHashSet;
use itertools::Itertools;
use tokio::sync::mpsc::Sender;
use tracing::error;

use crate::game::{ConnectName, ItemId, SlotId, SlotName};
use crate::pickle::value::Str;
use crate::proto;
use crate::proto::client::ItemsHandling;
use crate::proto::common::{Close, Control, ControlOrMessage};
use crate::proto::server::NetworkItem;
use crate::proto::server::{Message as ServerMessage, ReceivedItems};

#[derive(Clone)]
pub struct Client {
    server_message_tx: Sender<ControlOrMessage<Arc<ServerMessage>>>,
    pub address: SocketAddr,
    pub is_connected: bool,
    pub connect_name: ConnectName,
    pub slot_name: SlotName,
    pub slot_id: SlotId,
    pub wants_updates_for_keys: FnvHashSet<Str>,
    starting_inventory: FnvHashSet<ItemId>,
    items_handling: ItemsHandling,
    next_slot_item_index: usize,
    next_client_item_index: usize,
}

impl Client {
    pub fn new(
        address: SocketAddr,
        server_message_tx: Sender<ControlOrMessage<Arc<ServerMessage>>>,
    ) -> Self {
        Self {
            server_message_tx,
            address,
            is_connected: false,
            connect_name: ConnectName::new(),
            slot_name: SlotName::new(),
            slot_id: SlotId(-1),
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
        self.server_message_tx.send(message).await.ok();
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
