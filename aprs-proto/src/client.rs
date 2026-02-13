use aprs_value::Value;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

mod connect;
pub use connect::{Connect, ItemsHandling};

mod location_scouts;
pub use location_scouts::LocationScouts;

mod say;
pub use say::Say;

mod get;
pub use get::Get;

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

mod bounce;
pub use bounce::Bounce;

pub type Messages = SmallVec<[Message; 1]>;

#[derive(Serialize, Deserialize, Debug)]
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
    Bounce(Bounce),
    #[serde(untagged)]
    Unknown(Value),
}

impl From<Connect> for Message {
    fn from(value: Connect) -> Self {
        Message::Connect(value)
    }
}

impl From<Get> for Message {
    fn from(value: Get) -> Self {
        Message::Get(value)
    }
}

impl From<Set> for Message {
    fn from(value: Set) -> Self {
        Message::Set(value)
    }
}

impl From<SetNotify> for Message {
    fn from(value: SetNotify) -> Self {
        Message::SetNotify(value)
    }
}

impl From<Say> for Message {
    fn from(value: Say) -> Self {
        Message::Say(value)
    }
}

impl From<Sync> for Message {
    fn from(value: Sync) -> Self {
        Message::Sync(value)
    }
}

impl From<LocationScouts> for Message {
    fn from(value: LocationScouts) -> Self {
        Message::LocationScouts(value)
    }
}

impl From<LocationChecks> for Message {
    fn from(value: LocationChecks) -> Self {
        Message::LocationChecks(value)
    }
}

impl From<GetDataPackage> for Message {
    fn from(value: GetDataPackage) -> Self {
        Message::GetDataPackage(value)
    }
}

impl From<StatusUpdate> for Message {
    fn from(value: StatusUpdate) -> Self {
        Message::StatusUpdate(value)
    }
}

impl From<Bounce> for Message {
    fn from(value: Bounce) -> Self {
        Message::Bounce(value)
    }
}
