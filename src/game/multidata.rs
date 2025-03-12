use std::collections::BTreeMap;

use anyhow::Result;
use bstr::ByteSlice;
use itertools::Itertools;
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value as JsonValue};
use serde_value::Value;
use serde_with::{FromInto, serde_as};

use crate::game::{
    HashedGameData, LocationId, LocationInfo, MinimumVersions, NetworkSlot, PickledVersion,
    SeedName, ServerOptions, SlotId, SlotName, TeamAndSlot,
};
use crate::proto::common::NetworkVersion;

#[serde_as]
#[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
pub struct MultiData {
    pub slot_info: BTreeMap<SlotId, NetworkSlot>,
    #[serde(deserialize_with = "deserialize_pickle_slot_data")]
    pub slot_data: BTreeMap<SlotId, JsonValue>,
    pub connect_names: BTreeMap<SlotName, TeamAndSlot>,
    pub seed_name: SeedName,
    pub minimum_versions: MinimumVersions,
    pub server_options: ServerOptions,
    #[serde_as(as = "FromInto<PickledVersion>")]
    pub version: NetworkVersion,
    #[serde(rename = "datapackage")]
    pub data_package: BTreeMap<String, HashedGameData>,
    pub locations: BTreeMap<SlotId, BTreeMap<LocationId, LocationInfo>>,
    pub spheres: Vec<BTreeMap<SlotId, Vec<LocationId>>>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, Value>,
}

fn deserialize_pickle_slot_data<'de, D>(de: D) -> Result<BTreeMap<SlotId, JsonValue>, D::Error>
where
    D: Deserializer<'de>,
{
    let slot_data = <BTreeMap<SlotId, Value>>::deserialize(de)?;
    let slot_data = slot_data
        .into_iter()
        .map(|(slot_id, slot_data)| (slot_id, pickle_to_json(slot_data)))
        .collect::<BTreeMap<_, _>>();

    Ok(slot_data)
}

fn pickle_to_json(value: Value) -> JsonValue {
    match value {
        Value::Bool(value) => value.into(),
        Value::I64(value) => value.into(),
        Value::F64(value) => value.into(),
        Value::Bytes(value) => value.into(),
        Value::String(value) => value.into(),
        Value::Seq(values) => values
            .into_iter()
            .map(pickle_to_json)
            .collect::<Vec<_>>()
            .into(),
        Value::U8(n) => JsonValue::from(n),
        Value::U16(n) => JsonValue::from(n),
        Value::U32(n) => JsonValue::from(n),
        Value::U64(n) => JsonValue::from(n),
        Value::I8(n) => JsonValue::from(n),
        Value::I16(n) => JsonValue::from(n),
        Value::I32(n) => JsonValue::from(n),
        Value::F32(n) => JsonValue::from(n),
        Value::Char(c) => JsonValue::String(c.to_string()),
        Value::Unit => "()".into(),
        Value::Option(value) => value
            .map(|value| pickle_to_json(*value))
            .unwrap_or(JsonValue::Null),
        Value::Newtype(value) => pickle_to_json(*value),
        Value::Map(values) => values
            .into_iter()
            .map(|(key, value)| (pickle_key_to_string(key), pickle_to_json(value)))
            .collect::<Map<_, _>>()
            .into(),
    }
}

fn pickle_key_to_string(key: Value) -> String {
    match key {
        Value::Bool(value) => value.to_string(),
        Value::U8(value) => value.to_string(),
        Value::U16(value) => value.to_string(),
        Value::U32(value) => value.to_string(),
        Value::U64(value) => value.to_string(),
        Value::I8(value) => value.to_string(),
        Value::I16(value) => value.to_string(),
        Value::I32(value) => value.to_string(),
        Value::I64(value) => value.to_string(),
        Value::F32(value) => value.to_string(),
        Value::F64(value) => value.to_string(),
        Value::Char(value) => value.to_string(),
        Value::String(value) => value.to_string(),
        Value::Unit => "None".to_string(),
        Value::Option(value) => value
            .map(|value| pickle_key_to_string(*value))
            .unwrap_or_else(|| "None".into()),
        Value::Newtype(value) => pickle_key_to_string(*value),
        Value::Seq(values) => {
            let values = values.into_iter().map(pickle_key_to_string).join(", ");

            format!("({values})")
        }
        Value::Map(values) => {
            let values = values
                .into_iter()
                .map(|(key, value)| {
                    let key = pickle_key_to_string(key);
                    let value = pickle_key_to_string(value);

                    format!("{key}: {value}")
                })
                .join(", ");

            format!("{{{values}}}")
        }
        Value::Bytes(bytes) => bytes.as_bstr().to_string(),
    }
}
