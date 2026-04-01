use tokio::sync::oneshot;

use crate::net::ClientAddr;
use crate::server::client_id::ClientId;
use crate::server::control::Control;
use crate::server::{ClientMessages, ClientToServerConnection};

pub enum Event {
    ClientConnected(ClientAddr, oneshot::Sender<ClientToServerConnection>),
    ClientDisconnected(ClientId, ClientAddr),
    ClientMessages(ClientId, ClientMessages),
    ClientControl(ClientId, Control),
}
