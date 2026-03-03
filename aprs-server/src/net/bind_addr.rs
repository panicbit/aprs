#[cfg(unix)]
use std::path::PathBuf;
use std::str::FromStr;

#[cfg(not(unix))]
use color_eyre::eyre::bail;
use color_eyre::eyre::{self, Context, Result};

#[cfg(unix)]
use crate::net::UnixAddr;
use crate::net::{Bind, Listener, TcpAddr};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum BindAddr {
    Tcp(TcpAddr),
    #[cfg(unix)]
    Unix(UnixAddr),
}

impl Bind for BindAddr {
    type Listener = Listener;

    async fn bind(&self) -> Result<Listener> {
        Ok(match self {
            BindAddr::Tcp(tcp_addr) => Bind::bind(tcp_addr).await.map(Listener::Tcp)?,
            #[cfg(unix)]
            BindAddr::Unix(unix_addr) => Bind::bind(unix_addr).await.map(Listener::Unix)?,
        })
    }
}

impl<T> From<T> for BindAddr
where
    T: Into<TcpAddr>,
{
    fn from(value: T) -> Self {
        Self::Tcp(value.into())
    }
}

impl FromStr for BindAddr {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self> {
        const UNIX_PREFIX: &str = "unix:";

        if !s.starts_with(UNIX_PREFIX) {
            let addr = s
                .parse::<TcpAddr>()
                .context("failed to parse tcp/ip address")?;
            let addr = BindAddr::Tcp(addr);

            return Ok(addr);
        }

        #[cfg(unix)]
        {
            let s = &s[UNIX_PREFIX.len()..];
            let addr = PathBuf::from(s);
            let addr = UnixAddr::from(addr);
            let addr = BindAddr::Unix(addr);

            Ok(addr)
        }

        #[cfg(not(unix))]
        bail!("unix sockets are not implemented for this platform")
    }
}

#[cfg(test)]
mod tests {
    use crate::net::BindAddr;
    #[cfg(unix)]
    use crate::net::UnixAddr;

    #[test]
    fn parse_tcp_addr() {
        let expected = BindAddr::Tcp(([127, 0, 0, 1], 1234).into());
        let addr = "127.0.0.1:1234".parse::<BindAddr>().unwrap();

        assert_eq!(addr, expected);
    }

    #[test]
    #[should_panic]
    fn tcp_addr_must_have_port() {
        "127.0.0.1".parse::<BindAddr>().unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn parse_unix_addr() {
        let expected = BindAddr::Unix(UnixAddr::from("foo/bar.sock"));
        let addr = "unix:foo/bar.sock".parse::<BindAddr>().unwrap();

        assert_eq!(addr, expected);
    }
}
