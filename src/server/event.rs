use std::net::SocketAddr;

use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;

use crate::proto::client::Messages;

pub enum Event {
    ClientAccepted(SocketAddr, WebSocketStream<TcpStream>),
    ClientDisconnected(SocketAddr),
    ClientMessages(SocketAddr, Messages),
}
