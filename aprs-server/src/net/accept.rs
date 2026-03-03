use color_eyre::eyre::Result;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

use crate::net::{ClientAddr, TcpAddr};

pub trait Accept {
    type Stream: AsyncRead + AsyncWrite;

    fn accept(&self) -> impl Future<Output = Result<(Self::Stream, ClientAddr)>>;
}

impl Accept for TcpListener {
    type Stream = TcpStream;

    async fn accept(&self) -> Result<(Self::Stream, ClientAddr)> {
        let (stream, addr) = TcpListener::accept(self).await?;
        let addr = ClientAddr::Tcp(TcpAddr(addr));

        Ok((stream, addr))
    }
}

#[cfg(unix)]
impl Accept for UnixListener {
    type Stream = UnixStream;

    async fn accept(&self) -> Result<(Self::Stream, ClientAddr)> {
        let (stream, _) = UnixListener::accept(self).await?;
        let addr = ClientAddr::Unix;

        Ok((stream, addr))
    }
}
