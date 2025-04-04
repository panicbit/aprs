use std::sync::Arc;

use eyre::Result;
use futures::SinkExt;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite;
use tracing::debug;

use crate::proto::common::{Control, ControlOrMessage};
use crate::proto::server::Message;

// TODO: Arc messages
pub trait MessageSink {
    async fn send(&mut self, message: ControlOrMessage<Arc<Message>>) -> Result<()>;
    // TODO: maybe add ping and pong methods?
    async fn close(&mut self) -> Result<()>;
}

impl<S> MessageSink for tokio_tungstenite::WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    async fn send(&mut self, message: ControlOrMessage<Arc<Message>>) -> Result<()> {
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

        <Self as SinkExt<_>>::send(self, message).await?;

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        SinkExt::close(self).await?;
        Ok(())
    }
}
