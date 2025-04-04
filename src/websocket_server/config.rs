use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use crate::server;

#[derive(Clone)]
pub struct Config {
    pub listen_address: SocketAddr,
    pub state_path: PathBuf,
}

impl Config {
    pub fn with_listen_address(mut self, listen_address: SocketAddr) -> Self {
        self.listen_address = listen_address;
        self
    }

    pub fn with_listen_ip(mut self, listen_ip: IpAddr) -> Self {
        self.listen_address.set_ip(listen_ip);
        self
    }

    pub fn with_listen_port(mut self, listen_port: u16) -> Self {
        self.listen_address.set_port(listen_port);
        self
    }
}

impl From<Config> for server::Config {
    fn from(value: Config) -> Self {
        server::Config {
            state_path: value.state_path,
        }
    }
}
