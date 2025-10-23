use core::fmt;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, ops};

use bitflags::bitflags;
use byteorder::ReadBytesExt;
use eyre::{Context, ContextCompat, Result, ensure};
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use serde_tuple::Deserialize_tuple;
use serde_with::FromInto;
use serde_with::serde_as;
use sha1::{Digest, Sha1};

mod multidata;
pub use multidata::MultiData;

use crate::proto::common::NetworkVersion;
use crate::{FnvIndexMap, pickle};

#[derive(Debug)]
pub struct Game {
    pub multi_data: MultiData,
}

impl Game {
    pub fn load(path: impl AsRef<Path>) -> Result<Game> {
        let zip = fs::File::open(path).context("failed to open zip")?;
        let mut zip = zip::ZipArchive::new(zip).context("failed to read zip")?;
        let multi_data_filename = zip
            .file_names()
            .find(|name| name.ends_with(".archipelago"))
            .context("zip file does not contain a `.archipelago` file")?
            .to_owned();

        let mut multi_data = zip
            .by_name(&multi_data_filename)
            .with_context(|| format!("failed to read `{multi_data_filename}`"))?;

        let format_version = multi_data
            .read_u8()
            .context("failed to read format version")?;

        ensure!(
            format_version == 3,
            "unsupported format version `{format_version}`"
        );

        let mut multi_data = ZlibDecoder::new(multi_data);

        let multi_data = pickle::unpickle(&mut multi_data)?;

        let multi_data = MultiData::deserialize(multi_data)
            .with_context(|| format!("failed to deserialize `{multi_data_filename}`"))?;

        Ok(Game { multi_data })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ConnectName(pub String);

impl ConnectName {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ops::Deref for ConnectName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for ConnectName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<str> for ConnectName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for ConnectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotName(pub String);

impl SlotName {
    pub fn new() -> Self {
        Self(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ops::Deref for SlotName {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ops::DerefMut for SlotName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<str> for SlotName {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SlotName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotId(pub i64);

impl SlotId {
    pub const SERVER: SlotId = SlotId(0);

    pub fn is_server(&self) -> bool {
        self.0 == 0
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct TeamId(pub i64);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ItemId(pub i64);

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct LocationId(pub i64);

#[derive(Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SeedName(pub String);

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct ServerOptions {
    #[serde(rename = "password")]
    pub client_password: Option<String>,
    #[serde(rename = "server_password")]
    pub admin_password: Option<String>,
    pub release_mode: ReleaseMode,
    pub remaining_mode: RemainingMode,
    pub collect_mode: CollectMode,
    #[serde(flatten)]
    pub rest: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize_tuple, Debug, Hash, PartialEq, Eq, Copy, Clone)]
pub struct LocationInfo {
    pub item: ItemId,
    pub slot: SlotId,
    pub flags: u64,
}

#[repr(transparent)]
#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub struct ItemClassification(u32);

bitflags! {
    impl ItemClassification: u32 {
        const Progression = 0b001;
        const Useful = 0b010;
        const Trap = 0b100;
    }
}

impl ItemClassification {
    pub fn is_filler(&self) -> bool {
        self.is_empty()
    }

    pub fn is_progression(&self) -> bool {
        self.contains(ItemClassification::Useful)
    }

    pub fn is_useful(&self) -> bool {
        self.contains(ItemClassification::Useful)
    }

    pub fn is_trap(&self) -> bool {
        self.contains(ItemClassification::Trap)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct HashedGameData {
    pub checksum: String,
    #[serde(flatten)]
    pub game_data: GameData,
}

impl HashedGameData {
    pub fn checksum(&self) -> &str {
        &self.checksum
    }

    pub fn checksum_is_valid(&self) -> bool {
        let calculated = self.game_data.calculate_checksum();

        self.checksum.eq_ignore_ascii_case(&calculated)
    }
}

impl ops::Deref for HashedGameData {
    type Target = GameData;

    fn deref(&self) -> &Self::Target {
        &self.game_data
    }
}

impl ops::DerefMut for HashedGameData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.game_data
    }
}

// field order is significant for checksum calculation
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameData {
    // TODO: remove `serde(default)` once `proto` doesn't rely on this struct anymore
    #[serde(default)]
    pub item_name_groups: FnvIndexMap<String, Vec<String>>,
    pub item_name_to_id: FnvIndexMap<String, ItemId>,
    // TODO: remove `serde(default)` once `proto` doesn't rely on this struct anymore
    #[serde(default)]
    pub location_name_groups: FnvIndexMap<String, Vec<String>>,
    pub location_name_to_id: FnvIndexMap<String, LocationId>,
}

impl GameData {
    pub fn calculate_checksum(&self) -> String {
        let json = serde_json::to_string(&self).expect("failed to encode hash structure");
        let hash = Sha1::digest(&json);
        let hash = hex::encode(hash);

        hash
    }
}

#[derive(Deserialize_tuple, Debug)]
pub struct TeamAndSlot {
    pub team: TeamId,
    pub slot: SlotId,
}

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct MinimumVersions {
    #[serde_as(as = "FromInto<PickledVersion>")]
    pub server: NetworkVersion,
    #[serde_as(as = "BTreeMap<_, FromInto<PickledVersion>>")]
    pub clients: BTreeMap<SlotId, NetworkVersion>,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(deny_unknown_fields)]
/// This version struct is only used in the pickled data
pub struct PickledVersion {
    pub major: u32,
    pub minor: u32,
    /// This field only exists in the pickled data.
    pub patch: u32,
}

impl From<PickledVersion> for NetworkVersion {
    fn from(val: PickledVersion) -> Self {
        let PickledVersion {
            major,
            minor,
            patch,
        } = val;

        NetworkVersion {
            major,
            minor,
            build: patch,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetworkSlot {
    pub name: SlotName,
    pub game: String,
    pub r#type: SlotType,
    // TODO: implement for completeness some day maybe
    // https://github.com/ArchipelagoMW/Archipelago/blob/e00467c2a299623f630d5a3e68f35bc56ccaa8aa/NetUtils.py#L86
    pub group_members: serde_json::Value,
}

impl Serialize for NetworkSlot {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let NetworkSlot {
            name,
            game,
            r#type,
            group_members,
        } = &self;

        // The python client expects a "class" field in the json serialization
        #[derive(Serialize)]
        #[serde(tag = "class", rename = "NetworkSlot")]
        struct PythonNetworkSlot<'a> {
            pub name: &'a str,
            pub game: &'a str,
            pub r#type: &'a SlotType,
            // TODO: implement for completeness some day maybe
            // https://github.com/ArchipelagoMW/Archipelago/blob/e00467c2a299623f630d5a3e68f35bc56ccaa8aa/NetUtils.py#L86
            pub group_members: &'a serde_json::Value,
        }

        PythonNetworkSlot {
            name,
            game,
            r#type,
            group_members,
        }
        .serialize(ser)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[serde(transparent)]
pub struct SlotType(u32);

bitflags! {
    impl SlotType: u32 {
        const Player = 0b01;
        const Group = 0b10;
    }
}

impl SlotType {
    pub fn is_spectator(&self) -> bool {
        self.is_empty()
    }

    pub fn is_player(&self) -> bool {
        self.contains(SlotType::Player)
    }

    pub fn is_group(&self) -> bool {
        self.contains(SlotType::Group)
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum ReleaseMode {
    Disabled,
    Enabled,
    #[default]
    Auto,
    AutoEnabled,
    Goal,
}

// impl<'de> Deserialize<'de> for ReleaseMode {
//     fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // let s = String::deserialize(deserializer)?;
//         #[derive(Deserialize, Default, Debug)]
//         #[serde(rename_all = "kebab-case")]
//         pub enum Blah {
//             Disabled,
//             Enabled,
//             #[default]
//             Auto,
//             AutoEnabled,
//             Goal,
//         }

//         let v = Blah::deserialize(deserializer)?;

//         eprintln!("res = {v:?}");
//         todo!()
//     }
// }

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CollectMode {
    Disabled,
    Enabled,
    #[default]
    Auto,
    AutoEnabled,
    Goal,
}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum RemainingMode {
    Disabled,
    Enabled,
    #[default]
    Goal,
}
