use std::sync::Arc;

use serde::Serialize;

use crate::proto::common::{Close, Ping, Pong};

mod message_sink;
pub use message_sink::MessageSink;

mod message_stream;
pub use message_stream::MessageStream;

mod room_info;
pub use room_info::RoomInfo;

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

mod network_item;
pub use network_item::NetworkItem;

mod location_info;
pub use location_info::LocationInfo;

pub type GameName = String;

#[derive(Serialize)]
#[serde(tag = "cmd")]
pub enum Message {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected),
    Retrieved(Retrieved),
    LocationInfo(LocationInfo),
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

impl From<RoomInfo> for Arc<Message> {
    fn from(value: RoomInfo) -> Self {
        Arc::new(Message::RoomInfo(value))
    }
}

impl From<ConnectionRefused> for Arc<Message> {
    fn from(value: ConnectionRefused) -> Self {
        Arc::new(Message::ConnectionRefused(value))
    }
}

impl From<Connected> for Arc<Message> {
    fn from(value: Connected) -> Self {
        Arc::new(Message::Connected(value))
    }
}

impl From<Retrieved> for Arc<Message> {
    fn from(value: Retrieved) -> Self {
        Arc::new(Message::Retrieved(value))
    }
}

impl From<PrintJson> for Arc<Message> {
    fn from(value: PrintJson) -> Self {
        Arc::new(Message::PrintJson(value))
    }
}

impl From<Ping> for Arc<Message> {
    fn from(value: Ping) -> Self {
        Arc::new(Message::Ping(value))
    }
}

impl From<Pong> for Arc<Message> {
    fn from(value: Pong) -> Self {
        Arc::new(Message::Pong(value))
    }
}

impl From<Close> for Arc<Message> {
    fn from(value: Close) -> Self {
        Arc::new(Message::Close(value))
    }
}

impl From<Bounced> for Arc<Message> {
    fn from(value: Bounced) -> Self {
        Arc::new(Message::Bounced(value))
    }
}

impl From<LocationInfo> for Arc<Message> {
    fn from(value: LocationInfo) -> Self {
        Arc::new(Message::LocationInfo(value))
    }
}
