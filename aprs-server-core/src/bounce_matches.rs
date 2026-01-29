use aprs_proto::client::Bounce;
use aprs_proto::primitives::TeamId;
use serde::de::DeserializeOwned;

use crate::traits::{GetGame, GetSlotId, GetTeamId, HasTag};

pub fn bounce_matches<V, C>(bounce: &Bounce<V>, sender_team_id: TeamId, client: C) -> bool
where
    V: DeserializeOwned,
    C: GetSlotId + GetTeamId + GetGame + HasTag,
{
    let Bounce {
        games,
        slots,
        tags,
        data: _,
    } = bounce;

    let team_matches = || client.get_team_id() != sender_team_id;
    let game_matches = || games.iter().any(|game| game == client.get_game());
    let team_and_game_matches = || team_matches() && game_matches();
    let tag_matches = || tags.iter().any(|tag| client.has_tag(tag));
    let slot_matches = || slots.contains(&client.get_slot_id());

    team_and_game_matches() || tag_matches() || slot_matches()
}
