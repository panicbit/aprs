use std::ops::Bound::Included;

use crate::options::{DeathLink, Deserializer, MapAccess, Option};

#[derive(Default)]
//#[derive(Option)]
pub struct Options {
    /// Randomize Music
    //#[toggle(name = "Music Randomizer")]
    music_rando: bool,
    /// Randomize Entrances to Areas
    //#[option(name = "Area Randomizer")]
    area_rando: bool,
    /// Amount of Trinkets required to enter Areas. Set to 0 to disable artificial locks.
    //#[range(name = "Door Cost", start = 0, end = 3, default = 3)]
    door_cost: u32,
    /// Randomize which Area requires which set of DoorCost Trinkets
    //#[toggle(name = "Area Cost Randomizer")]
    area_cost: bool,
    death_link: DeathLink,
    /// Amount of Deaths to take before sending a DeathLink signal, for balancing difficulty
    //#[range(name = "Death Link Amnesty", start = 0, end = 30, default = 15)]
    death_link_amnesty: u32,
}

impl Option for Options {
    const NAME: &str = "V6 Options";
    const DESCRIPTION: &str = "";

    fn deserialize<D: Deserializer>(deserializer: D) -> Self {
        deserializer.deserialize_map(
            |map| Self {
                music_rando: map.get("music_rando").deserialize_toggle(false),
                area_rando: map.get("area_rando").deserialize_toggle(false),
                door_cost: map
                    .get("door_cost")
                    .deserialize_range(Included(0), Included(3), 3),
                area_cost: map.get("area_cost").deserialize_toggle(false),
                death_link: DeathLink::deserialize(map.get("death_link")),
                death_link_amnesty: map.get("death_link_amnesty").deserialize_range(
                    Included(0),
                    Included(30),
                    15,
                ),
            },
            <_>::default,
        )
    }
}
