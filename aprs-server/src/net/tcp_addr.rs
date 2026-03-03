use std::net::SocketAddr;
use std::str::FromStr;

use color_eyre::eyre::{self, Result};
use tokio::net::TcpListener;

use crate::net::Bind;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TcpAddr(pub SocketAddr);

impl Bind for TcpAddr {
    type Listener = TcpListener;

    async fn bind(&self) -> Result<Self::Listener> {
        Ok(TcpListener::bind(self.0).await?)
    }
}

impl FromStr for TcpAddr {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.parse::<SocketAddr>()?))
    }
}

impl<T> From<T> for TcpAddr
where
    T: Into<SocketAddr>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}
