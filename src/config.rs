use std::net::SocketAddr;
use std::path::PathBuf;

pub struct Config {
    pub general: General,
    pub websocket: WebSocket,
}

pub struct General {
    pub state_path: PathBuf,
}

pub struct WebSocket {
    pub listen_address: SocketAddr,
}
