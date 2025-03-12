use serde::{Deserialize, Serialize};
use serde_json::Value;
use smallvec::SmallVec;

use crate::proto::common::{Close, Ping, Pong};

mod connect;
pub use connect::Connect;

mod location_scouts;
pub use location_scouts::LocationScouts;

pub type Messages = SmallVec<[Message; 1]>;

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
pub enum Message {
    Connect(Connect),
    Get(Get),
    Say(Say),
    LocationScouts(LocationScouts),
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
