use std::net::SocketAddr;
use std::pin::pin;

use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::WebSocketStream;

use crate::game::SlotName;
use crate::proto::server::MessageSink;
use crate::proto::server::{Message as ServerMessage, MessageStream};
use crate::server::event::Event;

#[derive(Clone)]
pub struct Client {
    server_message_tx: Sender<ServerMessage>,
    pub address: SocketAddr,
    pub is_connected: bool,
    pub slot_name: SlotName,
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
        }
    }

    pub async fn send(&self, message: impl Into<ServerMessage>) {
        // TODO: handle overload situation, probably using timeout
        self.server_message_tx.send(message.into()).await.ok();
    }
}

async fn client_loop(
    stream: WebSocketStream<TcpStream>,
    address: SocketAddr,
    event_tx: Sender<Event>,
    mut server_message_rx: Receiver<ServerMessage>,
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
