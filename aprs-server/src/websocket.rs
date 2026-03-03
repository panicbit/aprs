use std::pin::pin;
use std::sync::Arc;

use color_eyre::eyre::{Context, Result, bail};
use format_serde_error::SerdeError;
use futures::SinkExt;
use smallvec::smallvec;
use tokio::select;
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::handshake::server::Callback;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{WebSocketStream, tungstenite};
use tracing::{debug, error};

use crate::net::{Accept, ClientAddr, Listener, Stream};
use crate::server::control::{Close, Control, ControlOrMessage, Ping, Pong};
use crate::server::{
    ClientId, ClientMessages, ClientToServerConnection, Event, ServerHandle, ServerMessage,
};

pub fn start(listener: Listener, server_handle: ServerHandle) {
    tokio::spawn(acceptor_loop(listener, server_handle.clone()));
}

async fn acceptor_loop(listener: Listener, server_handle: ServerHandle) {
    loop {
        select! {
            _ = server_handle.wait_for_stop() => {
                debug!("WS: acceptor loop shutting down due to server shutdown");
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

                let server_handle = server_handle.clone();

                tokio::spawn(async move {
                    if let Err(err) = handle_accept(stream, address.clone(), server_handle).await {
                        error!("Failed to accept client {address:?}: {err:?}");
                    }
                });
            }
        }
    }
}

async fn handle_accept(
    stream: Stream,
    address: ClientAddr,
    server_handle: ServerHandle,
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

    let client_id = ClientId::new();
    let connection = server_handle
        .client_accepted(client_id, address.clone())
        .await
        .context("server could not accept client")?;

    tokio::spawn(client_loop(
        client_id,
        stream,
        address,
        server_handle.clone(),
        connection,
    ));

    Ok(())
}

async fn client_loop(
    client_id: ClientId,
    stream: WebSocketStream<Stream>,
    address: ClientAddr,
    server_handle: ServerHandle,
    mut connection: ClientToServerConnection,
) {
    let mut stream = pin!(stream);
    let stream = &mut *stream;

    loop {
        let address = address.clone();

        select! {
            _ = server_handle.wait_for_stop() => return,
            server_message = connection.recv() => {
                let Some(server_message) = server_message else {
                    debug!("Server closed message channel to client");
                    return
                };

                // TODO: decouple sending and receiving
                if let Err(err) = send(stream, server_message).await {
                    error!("failed to send message to client: {err:?}");

                    connection.send(Event::ClientDisconnected(client_id, address)).await.ok();

                    return;
                }
            }
            client_messages = recv(stream) => {
                let event = match client_messages {
                    Ok(ControlOrMessage::Control(control)) => Event::ClientControl(client_id, control),
                    Ok(ControlOrMessage::Message(messages)) => Event::ClientMessages(client_id, messages),
                    Err(err) => {
                        error!("Failed to receive client messages: {err:?}");

                        connection.send(Event::ClientDisconnected(client_id, address)).await.ok();

                        return
                    }
                };

                connection.send(event).await.ok();
            }
        }
    }
}

async fn send(
    stream: &mut WebSocketStream<Stream>,
    message: ControlOrMessage<Arc<ServerMessage>>,
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

async fn recv(stream: &mut WebSocketStream<Stream>) -> Result<ControlOrMessage<ClientMessages>> {
    let message = stream.next().await.transpose()?;

    let Some(message) = message else {
        return Ok(smallvec![].into());
    };

    let message = match message {
        tungstenite::Message::Text(message) => {
            debug!("<<< {message}");
            let messages = serde_json::from_str::<ClientMessages>(message.as_str())
                .map_err(|err| SerdeError::new(message.to_string(), err))?;

            messages.into()
        }
        tungstenite::Message::Binary(message) => {
            debug!("<<< <binary>");
            let messages = serde_json::from_slice::<ClientMessages>(&message)?;

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
