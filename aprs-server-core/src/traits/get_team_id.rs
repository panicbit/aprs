use std::ops::Deref;

use aprs_proto::primitives::TeamId;

pub trait GetTeamId {
    fn get_team_id(&self) -> TeamId;
}

impl<T> GetTeamId for T
where
    T: Deref,
    T::Target: GetTeamId,
{
    fn get_team_id(&self) -> TeamId {
        self.deref().get_team_id()
    }
}
