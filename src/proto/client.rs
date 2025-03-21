use serde::{Deserialize, Serialize};
use serde_json::Value;
use smallvec::SmallVec;

use crate::proto::common::{Close, Ping, Pong};

mod connect;
pub use connect::{Connect, ItemsHandling};

mod location_scouts;
pub use location_scouts::LocationScouts;

mod set;
pub use set::{Set, SetOperation};

mod set_notify;
pub use set_notify::SetNotify;

mod location_checks;
pub use location_checks::LocationChecks;

mod sync;
pub use sync::Sync;

mod status_update;
pub use status_update::{ClientStatus, StatusUpdate};

mod get_data_package;
pub use get_data_package::GetDataPackage;

pub type Messages = SmallVec<[Message; 1]>;

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
pub enum Message {
    Connect(Connect),
    Get(Get),
    Set(Set),
    SetNotify(SetNotify),
    Say(Say),
    Sync(Sync),
    LocationScouts(LocationScouts),
    LocationChecks(LocationChecks),
    GetDataPackage(GetDataPackage),
    StatusUpdate(StatusUpdate),
    #[serde(skip)]
    Ping(Ping),
    #[serde(skip)]
    Pong(Pong),
    #[serde(skip)]
    Close(Close),
    #[serde(untagged)]
    Unknown(Value),
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Get {
    pub keys: SmallVec<[String; 1]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Say {
    pub text: String,
}
