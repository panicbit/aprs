use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;

#[pin_project(project = StreamProjection)]
pub enum Stream {
    Tcp(#[pin] TcpStream),
    #[cfg(unix)]
    Unix(#[pin] UnixStream),
}

impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProjection::Tcp(pin) => AsyncRead::poll_read(pin, cx, buf),
            #[cfg(unix)]
            StreamProjection::Unix(pin) => AsyncRead::poll_read(pin, cx, buf),
        }
    }
}

impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            StreamProjection::Tcp(pin) => AsyncWrite::poll_write(pin, cx, buf),
            #[cfg(unix)]
            StreamProjection::Unix(pin) => AsyncWrite::poll_write(pin, cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProjection::Tcp(pin) => AsyncWrite::poll_flush(pin, cx),
            #[cfg(unix)]
            StreamProjection::Unix(pin) => AsyncWrite::poll_flush(pin, cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match self.project() {
            StreamProjection::Tcp(pin) => AsyncWrite::poll_shutdown(pin, cx),
            #[cfg(unix)]
            StreamProjection::Unix(pin) => AsyncWrite::poll_shutdown(pin, cx),
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        match self.project() {
            StreamProjection::Tcp(pin) => AsyncWrite::poll_write_vectored(pin, cx, bufs),
            #[cfg(unix)]
            StreamProjection::Unix(pin) => AsyncWrite::poll_write_vectored(pin, cx, bufs),
        }
    }

    fn is_write_vectored(&self) -> bool {
        match self {
            Stream::Tcp(tcp_stream) => AsyncWrite::is_write_vectored(tcp_stream),
            #[cfg(unix)]
            Stream::Unix(unix_stream) => AsyncWrite::is_write_vectored(unix_stream),
        }
    }
}
