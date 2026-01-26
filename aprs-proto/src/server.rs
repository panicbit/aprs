use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

mod room_info;
pub use room_info::RoomInfo;

mod network_slot;
pub use network_slot::NetworkSlot;

mod slot_type;
pub use slot_type::SlotType;

mod permissions;
pub use permissions::{CommandPermission, Permissions, RemainingCommandPermission};

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

mod game_data;
pub use game_data::GameData;

mod hashed_game_data;
pub use hashed_game_data::HashedGameData;

#[derive(Serialize, Deserialize)]
#[serde(tag = "cmd")]
pub enum Message<V> {
    RoomInfo(RoomInfo),
    ConnectionRefused(ConnectionRefused),
    Connected(Connected<V>),
    Retrieved(Retrieved<V>),
    LocationInfo(LocationInfo),
    SetReply(SetReply<V>),
    ReceivedItems(ReceivedItems),
    RoomUpdate(RoomUpdate),
    DataPackage(DataPackage),
    #[serde(rename = "PrintJSON")]
    PrintJson(PrintJson),
    Bounced(Bounced<V>),
}

impl<V: DeserializeOwned> From<RoomInfo> for Arc<Message<V>> {
    fn from(value: RoomInfo) -> Self {
        Arc::new(Message::RoomInfo(value))
    }
}

impl<V: DeserializeOwned> From<ConnectionRefused> for Arc<Message<V>> {
    fn from(value: ConnectionRefused) -> Self {
        Arc::new(Message::ConnectionRefused(value))
    }
}

impl<V: DeserializeOwned> From<Connected<V>> for Arc<Message<V>> {
    fn from(value: Connected<V>) -> Self {
        Arc::new(Message::Connected(value))
    }
}

impl<V: DeserializeOwned> From<Retrieved<V>> for Arc<Message<V>> {
    fn from(value: Retrieved<V>) -> Self {
        Arc::new(Message::Retrieved(value))
    }
}

impl<V: DeserializeOwned> From<PrintJson> for Arc<Message<V>> {
    fn from(value: PrintJson) -> Self {
        Arc::new(Message::PrintJson(value))
    }
}

impl<V: DeserializeOwned> From<Bounced<V>> for Arc<Message<V>> {
    fn from(value: Bounced<V>) -> Self {
        Arc::new(Message::Bounced(value))
    }
}

impl<V: DeserializeOwned> From<LocationInfo> for Arc<Message<V>> {
    fn from(value: LocationInfo) -> Self {
        Arc::new(Message::LocationInfo(value))
    }
}

impl<V: DeserializeOwned> From<SetReply<V>> for Arc<Message<V>> {
    fn from(value: SetReply<V>) -> Self {
        Arc::new(Message::SetReply(value))
    }
}

impl<V: DeserializeOwned> From<ReceivedItems> for Arc<Message<V>> {
    fn from(value: ReceivedItems) -> Self {
        Arc::new(Message::ReceivedItems(value))
    }
}

impl<V: DeserializeOwned> From<RoomUpdate> for Arc<Message<V>> {
    fn from(value: RoomUpdate) -> Self {
        Arc::new(Message::RoomUpdate(value))
    }
}

impl<V: DeserializeOwned> From<DataPackage> for Arc<Message<V>> {
    fn from(value: DataPackage) -> Self {
        Arc::new(Message::DataPackage(value))
    }
}
