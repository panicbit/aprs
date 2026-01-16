use std::net::SocketAddr;

use crate::proto::client::Messages;
use crate::proto::common::Control;
use crate::server::Client;

pub enum Event {
    ClientAccepted(SocketAddr, Client),
    ClientDisconnected(SocketAddr),
    ClientMessages(SocketAddr, Messages),
    ClientControl(SocketAddr, Control),
}
