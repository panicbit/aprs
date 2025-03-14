use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Deserialize;
use serde_with::{FromInto, serde_as};

use crate::game::{
    GameData, HashedGameData, LocationId, LocationInfo, MinimumVersions, NetworkSlot,
    PickledVersion, SeedName, ServerOptions, SlotId, SlotName, TeamAndSlot,
};
use crate::pickle::Value;
use crate::proto::common::NetworkVersion;

#[serde_as]
#[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
pub struct MultiData {
    pub slot_info: Arc<BTreeMap<SlotId, NetworkSlot>>,
    pub slot_data: Arc<BTreeMap<SlotId, Value>>,
    pub connect_names: BTreeMap<SlotName, TeamAndSlot>,
    pub seed_name: SeedName,
    pub minimum_versions: MinimumVersions,
    pub server_options: ServerOptions,
    #[serde_as(as = "FromInto<PickledVersion>")]
    pub version: NetworkVersion,
    #[serde(rename = "datapackage")]
    pub data_package: Arc<BTreeMap<String, HashedGameData>>,
    pub locations: BTreeMap<SlotId, BTreeMap<LocationId, LocationInfo>>,
    pub spheres: Vec<BTreeMap<SlotId, Vec<LocationId>>>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, Value>,
}

impl MultiData {
    pub fn get_slot_id(&self, name: &str) -> Option<SlotId> {
        Some(self.connect_names.get(name)?.slot)
    }

    pub fn get_slot_info(&self, slot: SlotId) -> Option<&NetworkSlot> {
        self.slot_info.get(&slot)
    }

    pub fn get_locations(&self, slot: SlotId) -> Option<&BTreeMap<LocationId, LocationInfo>> {
        self.locations.get(&slot)
    }

    pub fn get_game_data(&self, game: &str) -> Option<&GameData> {
        Some(&self.data_package.get(game)?.game_data)
    }

    pub fn location_info(&self, slot: SlotId, location_id: LocationId) -> Option<&LocationInfo> {
        self.locations.get(&slot)?.get(&location_id)
    }

    pub fn slot_ids(&self) -> impl Iterator<Item = SlotId> {
        self.slot_info.keys().copied()
    }

    pub fn location_ids(&self, slot: SlotId) -> impl Iterator<Item = LocationId> {
        self.get_locations(slot)
            .into_iter()
            .flat_map(|map| map.keys())
            .copied()
    }
}

// TODO: replace this with a wrapper
// fn deserialize_pickle_slot_data<'de, D>(de: D) -> Result<Arc<BTreeMap<SlotId, Value>>, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let slot_data = <BTreeMap<SlotId, Value>>::deserialize(de)?;
//     let slot_data = slot_data
//         .into_iter()
//         .map(|(slot_id, slot_data)| (slot_id, slot_data))
//         // .map(|(slot_id, slot_data)| (slot_id, Arc::new(pickle_to_json(slot_data))))
//         .collect::<BTreeMap<_, _>>();

//     Ok(Arc::new(slot_data))
// }

// TODO: remove this
// fn pickle_to_json(value: Value) -> JsonValue {
//     match value {
//         Value::Bool(value) => value.into(),
//         Value::I64(value) => value.into(),
//         Value::F64(value) => value.into(),
//         Value::Bytes(value) => value.into(),
//         Value::String(value) => value.into(),
//         Value::Seq(values) => values
//             .into_iter()
//             .map(pickle_to_json)
//             .collect::<Vec<_>>()
//             .into(),
//         Value::U8(n) => JsonValue::from(n),
//         Value::U16(n) => JsonValue::from(n),
//         Value::U32(n) => JsonValue::from(n),
//         Value::U64(n) => JsonValue::from(n),
//         Value::U128(n) => JsonValue::from(Number::from_u128(n)),
//         Value::I8(n) => JsonValue::from(n),
//         Value::I16(n) => JsonValue::from(n),
//         Value::I32(n) => JsonValue::from(n),
//         Value::F32(n) => JsonValue::from(n),
//         Value::I128(n) => JsonValue::from(Number::from_i128(n)),
//         Value::Char(c) => JsonValue::String(c.to_string()),
//         Value::Unit => "()".into(),
//         Value::Option(value) => value
//             .map(|value| pickle_to_json(*value))
//             .unwrap_or(JsonValue::Null),
//         Value::Newtype(value) => pickle_to_json(*value),
//         Value::Map(values) => values
//             .into_iter()
//             .map(|(key, value)| (pickle_key_to_string(key), pickle_to_json(value)))
//             .collect::<Map<_, _>>()
//             .into(),
//     }
// }

// fn pickle_key_to_string(key: Value) -> String {
//     match key {
//         Value::Bool(value) => value.to_string(),
//         Value::U8(value) => value.to_string(),
//         Value::U16(value) => value.to_string(),
//         Value::U32(value) => value.to_string(),
//         Value::U64(value) => value.to_string(),
//         Value::U128(value) => value.to_string(),
//         Value::I8(value) => value.to_string(),
//         Value::I16(value) => value.to_string(),
//         Value::I32(value) => value.to_string(),
//         Value::I64(value) => value.to_string(),
//         Value::I128(value) => value.to_string(),
//         Value::F32(value) => value.to_string(),
//         Value::F64(value) => value.to_string(),
//         Value::Char(value) => value.to_string(),
//         Value::String(value) => value.to_string(),
//         Value::Unit => "null".to_string(),
//         Value::Option(value) => value
//             .map(|value| pickle_key_to_string(*value))
//             .unwrap_or_else(|| "null".into()),
//         Value::Newtype(value) => pickle_key_to_string(*value),
//         Value::Seq(values) => {
//             let values = values.into_iter().map(pickle_key_to_string).join(", ");

//             format!("({values})")
//         }
//         Value::Map(values) => {
//             let values = values
//                 .into_iter()
//                 .map(|(key, value)| {
//                     let key = pickle_key_to_string(key);
//                     let value = pickle_key_to_string(value);

//                     format!("{key}: {value}")
//                 })
//                 .join(", ");

//             format!("{{{values}}}")
//         }
//         Value::Bytes(bytes) => bytes.as_bstr().to_string(),
//     }
// }
