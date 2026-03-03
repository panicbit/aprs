use crate::net::TcpAddr;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ClientAddr {
    Tcp(TcpAddr),
    Unix,
}
