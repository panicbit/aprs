use std::borrow::Cow;
use std::net::SocketAddr;
use std::sync::Arc;

use eyre::{Context, Result, bail};
use format_serde_error::SerdeError;
use futures::stream::{SplitSink, SplitStream};
use futures::{Sink, SinkExt, Stream, StreamExt};
use ractor::{Actor, ActorCell, ActorProcessingErr, ActorRef, SupervisionEvent};
use smallvec::smallvec;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{WebSocketStream, tungstenite};
use tracing::{debug, error, info, warn};

use crate::config;
use crate::proto::client;
use crate::proto::common::{Close, Control, ControlOrMessage, Ping, Pong};
use crate::proto::server::{self, Message as ServerMessage};
use crate::server::{Client, ClientId, Event};

pub struct WebSocketServer;

impl Actor for WebSocketServer {
    type Msg = ();
    type State = Self;
    type Arguments = (ActorRef<Event>, config::WebSocket);

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        (server, config): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let listen_address = config.listen_address;
        let listener = TcpListener::bind(listen_address).await?;

        tokio::spawn(acceptor_loop(listener, server.clone()));

        Ok(Self {})
    }
}

async fn acceptor_loop(listener: TcpListener, server: ActorRef<Event>) {
    loop {
        select! {
            _ = server.wait(None) => {
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

                let server = server.clone();

                tokio::spawn(async move {
                    if let Err(err) = handle_accept(stream, address, server).await {
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
    server: ActorRef<Event>,
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

    let (websocket_client, _) = Actor::spawn(None, WebSocketClient, (stream, server.clone()))
        .await
        .context("failed to spawn websocket client actor")?;

    let client = Client::new(address, websocket_client);

    if let Err(err) = server.send_message(Event::ClientAccepted(client)) {
        debug!("Can't accept client: {err:?}");
    }

    Ok(())
}

async fn send(
    stream: &mut (impl Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin),
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
    stream: &mut (impl Stream<Item = tungstenite::Result<tungstenite::Message>> + Unpin),
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

struct WebSocketClient;

struct WebSocketClientState {
    stream: SplitSink<WebSocketStream<TcpStream>, tungstenite::Message>,
    server: ActorRef<Event>,
    client_id: ClientId,
}

impl Actor for WebSocketClient {
    type Msg = ControlOrMessage<Arc<ServerMessage>>;
    type State = WebSocketClientState;
    type Arguments = (WebSocketStream<TcpStream>, ActorRef<Event>);

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        (stream, server): Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (stream_write, stream_read) = stream.split();
        let client_id = ClientId::from(myself.get_id());

        tokio::spawn(web_socket_client_loop(
            stream_read,
            myself.get_cell(),
            server.clone(),
            client_id,
        ));

        Ok(WebSocketClientState {
            stream: stream_write,
            server,
            client_id,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        server_message: Self::Msg,
        this: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        // TODO: decouple sending and receiving
        if let Err(err) = send(&mut this.stream, server_message).await {
            error!("failed to send message to client: {err:?}");

            this.server
                .send_message(Event::ClientDisconnected(this.client_id))
                .ok();

            myself.kill();
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(_, _, reason) => {
                let reason = reason.map(Cow::from).unwrap_or(Cow::from("<unknown>"));

                info!("WebSocketClient stopped, reason: {reason}");

                myself.stop(None);
            }
            SupervisionEvent::ActorFailed(_, reason) => {
                error!("WebSocketClient failed, reason: {reason}");

                myself.stop(None);
            }
            _ => {}
        }

        Ok(())
    }
}

async fn web_socket_client_loop(
    mut stream: SplitStream<WebSocketStream<TcpStream>>,
    web_socket_client: ActorCell,
    server: ActorRef<Event>,
    client_id: ClientId,
) {
    loop {
        select! {
            _ = server.wait(None) => {
                web_socket_client.stop(Some("Server is shutting down".into()));
                return;
            }
            _ = web_socket_client.wait(None) => {
                warn!("Stopping `web_socket_client_loop`");
                return;
            },
            client_messages = recv(&mut stream) => {
                let event = match client_messages {
                    Ok(ControlOrMessage::Control(control)) => {
                        if control.is_close() {
                            web_socket_client.stop(Some("Client closed the connection".into()));
                        }

                        Event::ClientControl(client_id, control)
                    },
                    Ok(ControlOrMessage::Message(messages)) => Event::ClientMessages(client_id, messages),
                    Err(err) => {
                        error!("Failed to receive client messages: {err:?}");

                        server.send_message(Event::ClientDisconnected(client_id)).ok();

                        web_socket_client.stop(Some("Error receiving client message".into()));

                        return;
                    }
                };

                server.send_message(event).ok();

                continue
            }
        }
    }
}
