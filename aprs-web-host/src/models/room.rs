use std::ops::Deref;

use uuid::Uuid;

use crate::models::UserId;

pub struct Room {
    pub id: RoomId,
    pub owner: UserId,
}

#[derive(Copy, Clone, Debug)]
pub struct RoomId(Uuid);

impl RoomId {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Deref for RoomId {
    type Target = Uuid;

    fn deref(&self) -> &Uuid {
        &self.0
    }
}
