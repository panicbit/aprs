use color_eyre::Result;
use color_eyre::eyre::Context;
use tokio::sync::oneshot;

use crate::net::ClientAddr;
use crate::server::client_id::ClientId;
use crate::server::event::Event;
use crate::server::{ClientMessageSender, ClientToServerConnection};

#[derive(Clone)]
pub struct ServerHandle {
    pub(crate) client_message_sender: ClientMessageSender,
}

impl ServerHandle {
    pub async fn connect(&self, address: ClientAddr) -> Result<ClientToServerConnection> {
        let (reply_tx, reply_rx) = oneshot::channel();

        self.client_message_sender
            .send(Event::ClientConnected(address, reply_tx))
            .await?;

        let client_to_server_connection = reply_rx
            .await
            .context("failed to connect to server (internal)")?;

        Ok(client_to_server_connection)
    }

    pub async fn disconnect_client(&self, client_id: ClientId, address: ClientAddr) -> Result<()> {
        self.client_message_sender
            .send(Event::ClientDisconnected(client_id, address))
            .await?;
        Ok(())
    }

    pub fn wait_for_stop(&self) -> impl Future<Output = ()> {
        self.client_message_sender.closed()
    }
}
