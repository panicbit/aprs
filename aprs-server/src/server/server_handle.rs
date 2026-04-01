use color_eyre::Result;

use crate::net::ClientAddr;
use crate::server::client_id::ClientId;
use crate::server::event::Event;
use crate::server::{ClientMessageSender, ClientToServerConnection, Connection};

#[derive(Clone)]
pub struct ServerHandle {
    pub(crate) client_message_sender: ClientMessageSender,
}

impl ServerHandle {
    pub async fn connect(&self, address: ClientAddr) -> Result<ClientToServerConnection> {
        let client_id = ClientId::new();
        let (client_to_server_connection, server_to_client_connection) =
            Connection::new_pair(1_000, 1_000);

        self.client_message_sender
            .send(Event::ClientConnected(
                client_id,
                server_to_client_connection,
                address,
            ))
            .await?;

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
