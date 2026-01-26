use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Deserialize, Debug)]
pub struct StatusUpdate {
    pub status: ClientStatus,
}

#[derive(Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ClientStatus {
    Unknown = 0,
    Connected = 5,
    Ready = 10,
    Playing = 20,
    Goal = 30,
}
