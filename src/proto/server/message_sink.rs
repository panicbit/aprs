use std::sync::Arc;

use eyre::Result;
use futures::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;
use tracing::{debug, trace};

use crate::proto::server::Message;

// TODO: Arc messages
pub trait MessageSink {
    async fn send(&mut self, message: impl Into<Arc<Message>>) -> Result<()>;
    async fn close(&mut self) -> Result<()>;
}

impl<S> MessageSink for S
where
    S: Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin,
{
    async fn send(&mut self, message: impl Into<Arc<Message>>) -> Result<()> {
        let message = message.into();

        let message = match &*message {
            Message::Ping(ping) => tungstenite::Message::Ping(ping.0.clone()),
            Message::Pong(pong) => tungstenite::Message::Pong(pong.0.clone()),
            Message::Close(_close) => tungstenite::Message::Close(None),
            _ => {
                let json = serde_json::to_string(&[message])?;
                tungstenite::Message::text(json)
            }
        };

        debug!(">>> {message}");

        <Self as futures::sink::SinkExt<_>>::send(self, message).await?;

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        SinkExt::close(self).await?;
        Ok(())
    }
}
