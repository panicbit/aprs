use std::net::SocketAddr;

use crate::server::control::Control;
use crate::server::{Client, ClientMessages};

pub enum Event {
    ClientAccepted(SocketAddr, Client),
    ClientDisconnected(SocketAddr),
    ClientMessages(SocketAddr, ClientMessages),
    ClientControl(SocketAddr, Control),
}
