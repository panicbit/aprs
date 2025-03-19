use eyre::{Result, bail};
use format_serde_error::SerdeError;
use smallvec::smallvec;
use tokio_stream::Stream;
use tokio_tungstenite::tungstenite;
use tracing::debug;

use crate::proto::client;
use crate::proto::common::{Close, Ping, Pong};

pub trait MessageStream {
    async fn recv(&mut self) -> Result<client::Messages>;
}

impl<S> MessageStream for S
where
    S: Stream<Item = Result<tungstenite::Message, tungstenite::Error>> + Unpin,
{
    async fn recv(&mut self) -> Result<client::Messages> {
        let message = <Self as tokio_stream::StreamExt>::next(self)
            .await
            .transpose()?;

        let Some(message) = message else {
            return Ok(smallvec![]);
        };

        Ok(match message {
            tungstenite::Message::Text(message) => {
                debug!("<<< {message}");
                let messages = serde_json::from_str::<client::Messages>(message.as_str())
                    .map_err(|err| SerdeError::new(message.to_string(), err))?;

                messages
            }
            tungstenite::Message::Binary(message) => {
                debug!("<<< <binary>");
                let messages = serde_json::from_slice::<client::Messages>(&message)?;

                messages
            }
            tungstenite::Message::Ping(bytes) => {
                debug!("<<< ping");
                smallvec![Ping(bytes).into()]
            }
            tungstenite::Message::Pong(bytes) => {
                debug!("<<< pong");
                smallvec![Pong(bytes).into()]
            }
            tungstenite::Message::Close(_close_frame) => {
                smallvec![Close.into()]
            }
            tungstenite::Message::Frame(_frame) => bail!("received raw frame"),
        })
    }
}
