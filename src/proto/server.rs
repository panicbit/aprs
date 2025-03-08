use serde::Serialize;

use crate::proto::common::{Close, Ping, Pong};

mod message_sink;
pub use message_sink::MessageSink;

mod message_stream;
pub use message_stream::MessageStream;

mod room_info;
pub use room_info::RoomInfo;

mod network_version;
pub use network_version::NetworkVersion;

mod permissions;
pub use permissions::{CommandPermission, Permissions, RemainingCommandPermission};

mod sha1;
pub use sha1::Sha1;

mod time;
pub use time::Time;

mod retrieved;
pub use retrieved::Retrieved;

pub mod print_json;
pub use print_json::PrintJson;

mod bounced;
pub use bounced::Bounced;

mod connection_refused;
pub use connection_refused::{ConnectionError, ConnectionRefused};

mod connected;
pub use connected::Connected;

mod network_player;
pub use network_player::NetworkPlayer;

pub type GameName = String;

#[derive(Serialize)]
#[serde(tag = "cmd")]
pub enum Message {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected),
    Retrieved(Retrieved),
    #[serde(rename = "PrintJSON")]
    PrintJson(PrintJson),
    Bounced(Bounced),
    #[serde(skip)]
    Ping(Ping),
    #[serde(skip)]
    Pong(Pong),
    #[serde(skip)]
    Close(Close),
}

impl From<RoomInfo> for Message {
    fn from(value: RoomInfo) -> Self {
        Message::RoomInfo(value)
    }
}

impl From<ConnectionRefused> for Message {
    fn from(value: ConnectionRefused) -> Self {
        Message::ConnectionRefused(value)
    }
}

impl From<Connected> for Message {
    fn from(value: Connected) -> Self {
        Message::Connected(value)
    }
}

impl From<Retrieved> for Message {
    fn from(value: Retrieved) -> Self {
        Message::Retrieved(value)
    }
}

impl From<PrintJson> for Message {
    fn from(value: PrintJson) -> Self {
        Message::PrintJson(value)
    }
}

impl From<Ping> for Message {
    fn from(value: Ping) -> Self {
        Message::Ping(value)
    }
}

impl From<Pong> for Message {
    fn from(value: Pong) -> Self {
        Message::Pong(value)
    }
}

impl From<Close> for Message {
    fn from(value: Close) -> Self {
        Message::Close(value)
    }
}

impl From<Bounced> for Message {
    fn from(value: Bounced) -> Self {
        Message::Bounced(value)
    }
}
