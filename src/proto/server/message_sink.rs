use anyhow::Result;
use futures::{Sink, SinkExt};
use tokio_tungstenite::tungstenite;

use crate::proto::server::Message;

// TODO: Arc messages
pub trait MessageSink {
    async fn send(&mut self, message: impl Into<Message>) -> Result<()>;
    async fn close(&mut self) -> Result<()>;
}

impl<S> MessageSink for S
where
    S: Sink<tungstenite::Message, Error = tungstenite::Error> + Unpin,
{
    async fn send(&mut self, message: impl Into<Message>) -> Result<()> {
        let message = message.into();

        let message = match message {
            Message::Ping(ping) => tungstenite::Message::Ping(ping.0),
            Message::Pong(pong) => tungstenite::Message::Pong(pong.0),
            Message::Close(_close) => tungstenite::Message::Close(None),
            _ => {
                let json = serde_json::to_string_pretty(&[message])?;
                tungstenite::Message::text(json)
            }
        };

        eprintln!(">>> {message}");

        <Self as futures::sink::SinkExt<_>>::send(self, message).await?;

        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        SinkExt::close(self).await?;
        Ok(())
    }
}
