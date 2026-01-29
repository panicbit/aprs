use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::time::Instant;

use aprs_proto::common::NetworkVersion;
use aprs_proto::primitives::{ItemId, LocationId, SlotId, TeamId};
use bitflags::bitflags;
use byteorder::ReadBytesExt;
use color_eyre::eyre::{Context, ContextCompat, Result, bail, ensure};
use flate2::read::ZlibDecoder;
use serde::{Deserialize, Serialize};
use serde_tuple::Deserialize_tuple;
use serde_with::FromInto;
use serde_with::serde_as;
use sha1::{Digest, Sha1};
use tracing::info;

use crate::FnvIndexMap;

pub mod multidata;
pub use multidata::MultiData;

#[derive(Debug)]
pub struct Game {
    pub multi_data: MultiData,
}

impl Game {
    pub fn load_from_zip_or_bare(path: impl AsRef<Path>) -> Result<Game> {
        const INVALID_EXTENSION_ERROR: &str =
            "unknown extension (expected `.zip` or `.archipelago`)";

        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .context(INVALID_EXTENSION_ERROR)?;

        match extension {
            "zip" => Self::load_from_zip(path),
            "archipelago" => Self::load_from_bare(path),
            _ => bail!(INVALID_EXTENSION_ERROR),
        }
    }

    pub fn load_from_zip(path: impl AsRef<Path>) -> Result<Game> {
        let path = path.as_ref();
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

        Self::from_reader(&mut multi_data)
            .with_context(|| format!("failed to parse `{multi_data_filename}` in {path:?}"))
    }

    pub fn load_from_bare(path: impl AsRef<Path>) -> Result<Game> {
        let path = path.as_ref();
        let mut multi_data =
            File::open(path).with_context(|| format!("failed to open {path:?}"))?;

        Self::from_reader(&mut multi_data).with_context(|| format!("failed to parse {path:?}"))
    }

    pub fn from_reader<R: Read>(multi_data: &mut R) -> Result<Game> {
        let format_version = multi_data
            .read_u8()
            .context("failed to read format version")?;

        ensure!(
            format_version == 3,
            "unsupported format version `{format_version}`"
        );

        let decompression_start = Instant::now();
        let multi_data = {
            let mut data = Vec::new();
            ZlibDecoder::new(multi_data)
                .read_to_end(&mut data)
                .context("failed to decompress multi data")?;
            data
        };
        let decompression_time = decompression_start.elapsed();
        info!("Decompression finished in {:?}", decompression_time);

        let unpickle_start = Instant::now();
        let multi_data = aprs_pickle::unpickle(&multi_data, multidata::resolve_global)
            .context("failed to unpickle")?;
        let unpickle_time = unpickle_start.elapsed();
        info!("Unpickling finished in {:?}", unpickle_time);

        let deserialize_start = Instant::now();

        {}

        #[cfg(not(feature = "path_to_error"))]
        let multi_data = MultiData::deserialize(&multi_data);
        #[cfg(feature = "path_to_error")]
        let multi_data = serde_path_to_error::deserialize::<_, MultiData>(&multi_data);
        let multi_data = multi_data.context("failed to deserialize")?;
        let deserialize_time = deserialize_start.elapsed();
        info!("Deserializing finished in {:?}", deserialize_time);

        Ok(Game { multi_data })
    }
}

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

// field order MUST be alphabetical for correct checksum calculation.
// pickled game data contains extra fields compared to the network variant of it.
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
