use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

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

mod bounce;
pub use bounce::Bounce;

pub type Messages<V> = SmallVec<[Message<V>; 1]>;

#[derive(Deserialize, Debug)]
#[serde(tag = "cmd")]
pub enum Message<V> {
    Connect(Connect),
    Get(Get),
    Set(Set<V>),
    SetNotify(SetNotify),
    Say(Say),
    Sync(Sync),
    LocationScouts(LocationScouts),
    LocationChecks(LocationChecks),
    GetDataPackage(GetDataPackage),
    StatusUpdate(StatusUpdate),
    Bounce(Bounce<V>),
    #[serde(untagged)]
    Unknown(V),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Get {
    pub keys: SmallVec<[String; 1]>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Say {
    pub text: String,
}
