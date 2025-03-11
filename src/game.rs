use core::fmt;
use std::collections::BTreeMap;
use std::path::Path;
use std::{fs, ops};

use anyhow::{Context, Result, ensure};
use bitflags::bitflags;
use byteorder::ReadBytesExt;
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Deserializer, Serialize};
use serde_tuple::Deserialize_tuple;
use serde_with::FromInto;
use serde_with::serde_as;
use sha1::{Digest, Sha1};

mod multidata;
pub use multidata::MultiData;

use crate::FnvIndexMap;
use crate::proto::common::NetworkVersion;

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

        let value = crate::pickle::unpickle(&mut multi_data)?;
        // println!("{value:#?}");

        panic!("TEST END");

        // std::io::copy(
        //     &mut multi_data,
        //     &mut std::fs::File::create("test.pickled").unwrap(),
        // )
        // .unwrap();
        // panic!("STOP HERE");

        let decode_options = serde_pickle::DeOptions::new()
            // .keep_restore_state()
            // .replace_recursive_structures()
            ;

        let de = &mut serde_pickle::Deserializer::new(multi_data, decode_options);
        let multi_data = serde_path_to_error::deserialize::<_, MultiData>(de)
            .with_context(|| format!("failed to deserialize `{multi_data_filename}`"))?;

        // println!("{multi_data:#?}");

        Ok(Game { multi_data })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotName(pub String);

impl SlotName {
    pub fn empty() -> Self {
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

impl fmt::Display for SlotName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct SlotId(pub i64);

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
    pub rest: BTreeMap<String, serde_pickle::Value>,
}

#[derive(Deserialize_tuple, Debug)]
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
#[derive(Deserialize, Serialize, Debug)]
pub struct GameData {
    pub item_name_groups: FnvIndexMap<String, Vec<String>>,
    pub item_name_to_id: FnvIndexMap<String, ItemId>,
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

#[derive(Deserialize, Debug)]
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

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "class")]
pub struct NetworkSlot {
    pub name: String,
    pub game: String,
    pub r#type: SlotType,
    // TODO: implement for completeness some day maybe
    // https://github.com/ArchipelagoMW/Archipelago/blob/e00467c2a299623f630d5a3e68f35bc56ccaa8aa/NetUtils.py#L86
    pub group_members: serde_json::Value,
}

impl<'de> Deserialize<'de> for NetworkSlot {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (name, game, (r#type,), group_members) = <_>::deserialize(de)?;

        Ok(Self {
            name,
            game,
            r#type,
            group_members,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
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
