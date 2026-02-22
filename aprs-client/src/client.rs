use std::collections::VecDeque;

use aprs_proto::{client, server};
use eyre::{Context, ContextCompat, Result, bail};
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use itertools::Itertools;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Utf8Bytes;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite};

type WSStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct Client {
    message_sink: SplitSink<WSStream, tungstenite::Message>,
    message_rx: mpsc::Receiver<Result<tungstenite::Message>>,
    messages: VecDeque<server::Message>,
}

impl Client {
    pub async fn connect(addr: String) -> Result<Self> {
        let (stream, _resp) = tokio_tungstenite::connect_async(addr)
            .await
            .context("failed to connect")?;

        let (message_sink, mut message_stream) = stream.split();
        let (message_tx, message_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            while let Some(message) = message_stream.next().await {
                let message = message.context("failed to receive message");
                message_tx
                    .send(message)
                    .await
                    .context("failed to send received message")?;
            }

            eyre::Ok(())
        });

        Ok(Self {
            message_sink,
            message_rx,
            messages: VecDeque::new(),
        })
    }

    pub async fn send(&mut self, msg: impl Into<client::Message>) -> Result<()> {
        let messages = [msg.into()];

        self.send_many(&messages)
            .await
            .context("failed to send message")
    }

    pub async fn send_many(&mut self, messages: &[client::Message]) -> Result<()> {
        let messages =
            serde_json::to_string(&messages).context("failed to encode message as json")?;
        let messages = tungstenite::Message::Text(messages.into());

        self.message_sink
            .send(messages)
            .await
            .context("failed to send messages")?;

        Ok(())
    }

    pub async fn login(&mut self, connect: client::Connect) -> Result<server::Connected> {
        self.send(connect)
            .await
            .context("failed to send Connect packet")?;

        self.wait_for_connected()
            .await
            .context("failed to receive Connected packet")
    }

    async fn wait_for_connected(&mut self) -> Result<server::Connected> {
        loop {
            let message = self.next_message().await?.context("disconnected")?;

            match message {
                server::Message::Connected(connected) => return Ok(connected),
                server::Message::ConnectionRefused(connection_refused) => bail!(
                    "connection refused:\n{}",
                    connection_refused
                        .errors
                        .iter()
                        .map(|err| format!("- {err:?}"))
                        .join("\n")
                ),
                server::Message::RoomInfo(_) => continue,
                server::Message::Retrieved(_) => continue,
                server::Message::LocationInfo(_) => continue,
                server::Message::SetReply(_) => continue,
                server::Message::ReceivedItems(_) => continue,
                server::Message::RoomUpdate(_) => continue,
                server::Message::DataPackage(_) => continue,
                server::Message::PrintJson(_) => continue,
                server::Message::Bounced(_) => continue,
                server::Message::InvalidPacket(invalid_packet) => {
                    bail!("Invalid packet: {:?}", invalid_packet)
                }
            }
        }
    }

    pub async fn next_message(&mut self) -> Result<Option<server::Message>> {
        loop {
            let Some(message) = self.messages.pop_front() else {
                let messages = self
                    .recv_messages()
                    .await
                    .context("failed to receive more messages")?;

                let Some(messages) = messages else {
                    return Ok(None);
                };

                self.messages.extend(messages);

                continue;
            };

            return Ok(Some(message));
        }
    }

    async fn recv_messages(&mut self) -> Result<Option<Vec<server::Message>>> {
        let Some(message) = self.recv_text_message().await? else {
            return Ok(None);
        };

        let mut de = serde_json::Deserializer::from_str(message.as_str());
        let messages = serde_path_to_error::deserialize::<_, Vec<server::Message>>(&mut de)
            .context("failed to deserialize server message")?;

        Ok(Some(messages))
    }

    async fn recv_text_message(&mut self) -> Result<Option<Utf8Bytes>> {
        loop {
            let Some(message) = self.message_rx.recv().await else {
                return Ok(None);
            };
            let message = message.context("failed to receive message")?;

            match message {
                tungstenite::Message::Text(message) => return Ok(Some(message)),
                tungstenite::Message::Binary(_) => bail!("Unexpected Binary"),
                tungstenite::Message::Ping(bytes) => {
                    self.message_sink
                        .send(tungstenite::Message::Pong(bytes))
                        .await
                        .context("failed to send Pong")?;
                }
                tungstenite::Message::Pong(_) => bail!("Unexpected Pong"),
                tungstenite::Message::Close(_) => return Ok(None),
                tungstenite::Message::Frame(_) => bail!("Unexpected Frame"),
            };
        }
    }
}
