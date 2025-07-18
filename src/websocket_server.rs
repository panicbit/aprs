use std::net::SocketAddr;
use std::pin::pin;
use std::sync::Arc;

use eyre::{Result, bail};
use format_serde_error::SerdeError;
use futures::SinkExt;
use smallvec::smallvec;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{WebSocketStream, tungstenite};
use tracing::{debug, error};

use crate::game::MultiData;
use crate::proto::client;
use crate::proto::common::{Close, Control, ControlOrMessage, Ping, Pong};
use crate::proto::server;
use crate::server::{Client, ClientId, Event, Server};

mod config;
pub use config::Config;

pub struct WebsocketServer {
    server: Server,
    config: Config,
    tx: Sender<Event>,
}

impl WebsocketServer {
    pub fn new(config: Config, multi_data: MultiData) -> Result<Self> {
        let (tx, rx) = mpsc::channel(10_000);

        let server = Server::new(config.clone().into(), multi_data, rx)?;

        Ok(Self { server, config, tx })
    }

    pub async fn run(self) -> Result<()> {
        let listen_address = self.config.listen_address;
        let listener = TcpListener::bind(listen_address).await?;

        tokio::spawn(acceptor_loop(listener, self.tx.clone()));

        self.server.run().await
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

    let (server_message_tx, server_message_rx) = mpsc::channel(1_000);
    let client = Client::new(address, server_message_tx);

    tokio::spawn(client_loop(
        stream,
        client.id(),
        event_tx.clone(),
        server_message_rx,
    ));

    if event_tx.send(Event::ClientAccepted(client)).await.is_err() {
        debug!("Can't accept client, event channel is closed");
    }

    Ok(())
}

async fn client_loop(
    stream: WebSocketStream<TcpStream>,
    client_id: ClientId,
    event_tx: Sender<Event>,
    mut server_message_rx: Receiver<ControlOrMessage<Arc<server::Message>>>,
) {
    let mut stream = pin!(stream);
    let stream = &mut *stream;

    loop {
        select! {
            _ = event_tx.closed() => return,
            server_message = server_message_rx.recv() => {
                let Some(server_message) = server_message else {
                    debug!("Server closed message channel to client");
                    return
                };

                // TODO: decouple sending and receiving
                if let Err(err) = send(stream, server_message).await {
                    error!("failed to send message to client: {err:?}");

                    event_tx.send(Event::ClientDisconnected(client_id)).await.ok();

                    return;
                }
            }
            client_messages = recv(stream) => {
                let event = match client_messages {
                    Ok(ControlOrMessage::Control(control)) => Event::ClientControl(client_id, control),
                    Ok(ControlOrMessage::Message(messages)) => Event::ClientMessages(client_id, messages),
                    Err(err) => {
                        error!("Failed to receive client messages: {err:?}");

                        event_tx.send(Event::ClientDisconnected(client_id)).await.ok();

                        return
                    }
                };

                event_tx.send(event).await.ok();
            }
        }
    }
}

async fn send(
    stream: &mut WebSocketStream<TcpStream>,
    message: ControlOrMessage<Arc<server::Message>>,
) -> Result<()> {
    let message = match message {
        ControlOrMessage::Control(Control::Ping(ping)) => {
            tungstenite::Message::Ping(ping.0.clone())
        }
        ControlOrMessage::Control(Control::Pong(pong)) => {
            tungstenite::Message::Pong(pong.0.clone())
        }
        ControlOrMessage::Control(Control::Close(_close)) => tungstenite::Message::Close(None),
        ControlOrMessage::Message(message) => {
            let json = serde_json::to_string(&[message])?;
            tungstenite::Message::text(json)
        }
    };

    debug!(">>> {message}");

    stream.send(message).await?;

    Ok(())
}

async fn recv(
    stream: &mut WebSocketStream<TcpStream>,
) -> Result<ControlOrMessage<client::Messages>> {
    let message = stream.next().await.transpose()?;

    let Some(message) = message else {
        return Ok(smallvec![].into());
    };

    let message = match message {
        tungstenite::Message::Text(message) => {
            debug!("<<< {message}");
            let messages = serde_json::from_str::<client::Messages>(message.as_str())
                .map_err(|err| SerdeError::new(message.to_string(), err))?;

            messages.into()
        }
        tungstenite::Message::Binary(message) => {
            debug!("<<< <binary>");
            let messages = serde_json::from_slice::<client::Messages>(&message)?;

            messages.into()
        }
        tungstenite::Message::Ping(bytes) => {
            // TODO: allow passing on Ping to the server
            debug!("<<< ping ({} bytes)", bytes.len());

            Ping(bytes).into()
        }
        tungstenite::Message::Pong(bytes) => {
            // TODO: allow passing on Pong to the server
            debug!("<<< pong ({} bytes)", bytes.len());

            Pong(bytes).into()
        }
        tungstenite::Message::Close(_close_frame) => {
            debug!("<<< close");

            Close.into()
        }
        tungstenite::Message::Frame(_frame) => {
            error!("BUG: received raw frame");
            bail!("BUG: received raw frame");
        }
    };

    Ok(message)
}
