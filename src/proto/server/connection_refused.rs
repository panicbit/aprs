use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ConnectionRefused {
    pub errors: Vec<ConnectionError>,
}

impl ConnectionRefused {
    pub fn invalid_slot() -> Self {
        ConnectionError::InvalidSlot.into()
    }
    pub fn invalid_game() -> Self {
        ConnectionError::InvalidGame.into()
    }
    pub fn incompatible_version() -> Self {
        ConnectionError::IncompatibleVersion.into()
    }
    pub fn invalid_password() -> Self {
        ConnectionError::InvalidPassword.into()
    }
    pub fn invalid_items_handling() -> Self {
        ConnectionError::InvalidItemsHandling.into()
    }
}

impl From<ConnectionError> for ConnectionRefused {
    fn from(value: ConnectionError) -> Self {
        Self {
            errors: vec![value],
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum ConnectionError {
    InvalidSlot,
    InvalidGame,
    IncompatibleVersion,
    InvalidPassword,
    InvalidItemsHandling,
    #[serde(untagged)]
    Unknown(String),
}
