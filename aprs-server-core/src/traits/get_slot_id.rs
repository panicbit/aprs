use std::ops::Deref;

use aprs_proto::primitives::SlotId;

pub trait GetSlotId {
    fn get_slot_id(&self) -> SlotId;
}

impl<T> GetSlotId for T
where
    T: Deref,
    T::Target: GetSlotId,
{
    fn get_slot_id(&self) -> SlotId {
        self.deref().get_slot_id()
    }
}
