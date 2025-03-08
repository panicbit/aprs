use bytes::Bytes;
use std::ops;

pub mod network_version;
pub use network_version::NetworkVersion;

#[derive(Debug, Clone)]
pub struct Ping(pub Bytes);

impl ops::Deref for Ping {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Pong(pub Bytes);

impl ops::Deref for Pong {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Close;
