use eyre::{Result, bail};
use format_serde_error::SerdeError;
use smallvec::smallvec;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite;
use tracing::{debug, error};

use crate::proto::client;
use crate::proto::common::{Close, ControlOrMessage, Ping, Pong};

pub trait MessageStream {
    async fn recv(&mut self) -> Result<ControlOrMessage<client::Messages>>;
}

impl<S> MessageStream for tokio_tungstenite::WebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    async fn recv(&mut self) -> Result<ControlOrMessage<client::Messages>> {
        let message = <Self as StreamExt>::next(self).await.transpose()?;

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
}
