use color_eyre::eyre::Result;
use tokio::net::TcpListener;
#[cfg(unix)]
use tokio::net::UnixListener;

use crate::net::{Accept, Stream};

pub enum Listener {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix(UnixListener),
}

impl Accept for Listener {
    type Stream = Stream;

    async fn accept(&self) -> Result<(Self::Stream, super::ClientAddr)> {
        match self {
            Listener::Tcp(tcp_listener) => {
                let (stream, addr) = Accept::accept(tcp_listener).await?;
                Ok((Stream::Tcp(stream), addr))
            }
            #[cfg(unix)]
            Listener::Unix(unix_listener) => {
                let (stream, addr) = Accept::accept(unix_listener).await?;
                Ok((Stream::Unix(stream), addr))
            }
        }
    }
}
