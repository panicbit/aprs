use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

use crate::proto::common::NetworkVersion;
use crate::proto::server::{GameName, Permissions, Time};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoomInfo {
    /// Version the server is running
    pub version: NetworkVersion,
    /// Version of the generator of the multiworld
    pub generator_version: NetworkVersion,
    /// Denotes special features or capabilities the server is capable of.
    /// Example: `WebHost`
    pub tags: Vec<String>,
    /// Denotes whether a password is required
    pub password: bool,
    /// Permissions for various commands
    pub permissions: Permissions,
    /// The percentage of checks a player needs to receive a hint from the server.
    pub hint_cost: u8,
    /// The amount of hint points a player receives per item/location check completed.
    pub location_check_points: u32,
    /// List of games present in the multiworld.
    pub games: Vec<GameName>,
    /// SHA-1 hashes of the game's data packages.
    /// Newer clients use it to invalidate their data package cache.
    pub datapackage_checksums: FnvHashMap<GameName, String>,
    /// A name that unique identifies the seed used for generation.
    /// Note: This is not necessarily the same as the seed itself.
    pub seed_name: String,
    /// Current server time in seconds since the Unix epoch.
    pub time: Time,
}
