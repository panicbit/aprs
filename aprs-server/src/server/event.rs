use std::net::SocketAddr;

use crate::server::control::Control;
use crate::server::{
    ClientMessages, ServerToClientConnection,
};

pub enum Event {
    ClientAccepted(ServerToClientConnection, SocketAddr),
    ClientDisconnected(SocketAddr),
    ClientMessages(SocketAddr, ClientMessages),
    ClientControl(SocketAddr, Control),
}
