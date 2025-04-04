use std::sync::Arc;

use serde::{Deserialize, Serialize};

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

mod set_reply;
pub use set_reply::SetReply;

mod received_items;
pub use received_items::ReceivedItems;

mod room_update;
pub use room_update::RoomUpdate;

mod data_package;
pub use data_package::{DataPackage, DataPackageData};

pub type GameName = String;

#[derive(Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Message {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected),
    Retrieved(Retrieved),
    LocationInfo(LocationInfo),
    SetReply(SetReply),
    ReceivedItems(ReceivedItems),
    RoomUpdate(RoomUpdate),
    DataPackage(DataPackage),
    #[serde(rename = "PrintJSON")]
    PrintJson(PrintJson),
    Bounced(Bounced),
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

impl From<SetReply> for Arc<Message> {
    fn from(value: SetReply) -> Self {
        Arc::new(Message::SetReply(value))
    }
}

impl From<ReceivedItems> for Arc<Message> {
    fn from(value: ReceivedItems) -> Self {
        Arc::new(Message::ReceivedItems(value))
    }
}

impl From<RoomUpdate> for Arc<Message> {
    fn from(value: RoomUpdate) -> Self {
        Arc::new(Message::RoomUpdate(value))
    }
}

impl From<DataPackage> for Arc<Message> {
    fn from(value: DataPackage) -> Self {
        Arc::new(Message::DataPackage(value))
    }
}
