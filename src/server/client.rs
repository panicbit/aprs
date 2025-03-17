use std::net::SocketAddr;
use std::pin::pin;
use std::sync::Arc;

use fnv::FnvHashSet;
use itertools::Itertools;
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::WebSocketStream;

use crate::game::{ItemId, SlotId, SlotName};
use crate::pickle::value::Str;
use crate::proto;
use crate::proto::server::{Message as ServerMessage, MessageStream, ReceivedItems};
use crate::proto::server::{MessageSink, NetworkItem};
use crate::server::event::Event;

#[derive(Clone)]
pub struct Client {
    server_message_tx: Sender<Arc<ServerMessage>>,
    pub address: SocketAddr,
    pub is_connected: bool,
    pub slot_name: SlotName,
    pub slot_id: SlotId,
    pub wants_updates_for_keys: FnvHashSet<Str>,
    items_handling: ItemsHandling,
    next_slot_item_index: usize,
    next_client_item_index: usize,
}

impl Client {
    pub fn new(
        address: SocketAddr,
        stream: WebSocketStream<TcpStream>,
        event_tx: Sender<Event>,
    ) -> Self {
        let (server_message_tx, server_message_rx) = mpsc::channel(1_000);

        tokio::spawn(client_loop(stream, address, event_tx, server_message_rx));

        Self {
            server_message_tx,
            address,
            is_connected: false,
            slot_name: SlotName::empty(),
            slot_id: SlotId(-1),
            wants_updates_for_keys: FnvHashSet::default(),
            items_handling: ItemsHandling::NoItems,
            next_slot_item_index: 0,
            next_client_item_index: 0,
        }
    }

    pub async fn send(&self, message: impl Into<Arc<ServerMessage>>) {
        // TODO: handle overload situation, probably using timeout
        self.server_message_tx.send(message.into()).await.ok();
    }

    pub fn set_items_handling(&mut self, items_handling: proto::client::ItemsHandling) {
        eprintln!("Incoming items handling: 0b{items_handling:03b}");
        let new_items_handling = ItemsHandling::from(items_handling);
        eprintln!("Converted items handling: {new_items_handling:?}");

        if new_items_handling == self.items_handling {
            return;
        }

        self.items_handling = new_items_handling;
        self.reset_received_items();
    }

    pub fn reset_received_items(&mut self) {
        self.next_slot_item_index = 0;
        self.next_client_item_index = 0;
    }

    pub async fn sync_items(&mut self, slot_items: &[NetworkItem], starting_inventory: &[ItemId]) {
        dbg!(slot_items, starting_inventory);

        let Some(missing_items) = slot_items.get(self.next_slot_item_index..) else {
            eprintln!("BUG: next_slot_item_index out of bounds");
            return;
        };

        dbg!(missing_items);

        let client_index = self.next_client_item_index;

        let missing_items = match self.items_handling {
            ItemsHandling::NoItems => vec![],
            ItemsHandling::All => missing_items.to_owned(),
            ItemsHandling::OwnWorld => missing_items
                .iter()
                .filter(|item| item.player == self.slot_id)
                .copied()
                .collect_vec(),
            ItemsHandling::StartingInventory => missing_items
                .iter()
                .filter(|item| item.player == self.slot_id)
                .filter(|item| starting_inventory.contains(&item.item))
                .copied()
                .collect_vec(),
        };

        dbg!(&missing_items);

        self.next_client_item_index += missing_items.len();
        self.next_slot_item_index = slot_items.len();

        if missing_items.is_empty() {
            return;
        }

        self.send(ReceivedItems {
            index: client_index,
            items: missing_items.to_owned(),
        })
        .await;
    }
}

async fn client_loop(
    stream: WebSocketStream<TcpStream>,
    address: SocketAddr,
    event_tx: Sender<Event>,
    mut server_message_rx: Receiver<Arc<ServerMessage>>,
) {
    let mut stream = pin!(stream);
    let stream = &mut *stream;

    loop {
        select! {
            _ = event_tx.closed() => return,
            server_message = server_message_rx.recv() => {
                let Some(server_message) = server_message else {
                    eprintln!("Server closed message channel to client");
                    return
                };

                // TODO: decouple sending and receiving
                if let Err(err) = stream.send(server_message).await {
                    eprintln!("failed to send message to client: {err:?}");

                    event_tx.send(Event::ClientDisconnected(address)).await.ok();

                    return;
                }
            }
            client_messages = stream.recv() => {
                let client_messages = match client_messages {
                    Ok(client_messages) => client_messages,
                    Err(err) => {
                        eprintln!("Failed to receive client messages: {err:?}");

                        event_tx.send(Event::ClientDisconnected(address)).await.ok();

                        return
                    }
                };

                event_tx.send(Event::ClientMessages(address, client_messages)).await.ok();
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ItemsHandling {
    NoItems,
    All,
    OwnWorld,
    StartingInventory,
}

impl From<proto::client::ItemsHandling> for ItemsHandling {
    fn from(value: proto::client::ItemsHandling) -> Self {
        if value.is_remote() {
            if value.is_own_world_only() {
                return Self::OwnWorld;
            } else if value.is_starting_inventory_only() {
                return Self::StartingInventory;
            } else {
                return Self::All;
            }
        }

        Self::NoItems
    }
}
