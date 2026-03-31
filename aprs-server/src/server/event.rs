use crate::net::ClientAddr;
use crate::server::client_id::ClientId;
use crate::server::control::Control;
use crate::server::{ClientMessages, ServerToClientConnection};

pub enum Event {
    ClientConnected(ClientId, ServerToClientConnection, ClientAddr),
    ClientDisconnected(ClientId, ClientAddr),
    ClientMessages(ClientId, ClientMessages),
    ClientControl(ClientId, Control),
}
