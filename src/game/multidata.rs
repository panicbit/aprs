use std::collections::BTreeMap;

use anyhow::{Context, Result};
use bstr::ByteSlice;
use serde::{de, Deserialize, Deserializer};
use serde_json::{Map, Value};
use serde_pickle::{HashableValue, Value as PickleValue};

use crate::game::{
    HashedGameData, LocationId, LocationInfo, MinimumVersions, NetworkSlot, PickledVersion,
    SeedName, ServerOptions, SlotId, SlotName, TeamAndSlot,
};

#[derive(Deserialize, Debug)]
// #[serde(deny_unknown_fields)]
pub struct MultiData {
    pub slot_info: BTreeMap<SlotId, NetworkSlot>,
    #[serde(deserialize_with = "deserialize_pickle_slot_data")]
    pub slot_data: BTreeMap<SlotId, Value>,
    pub connect_names: BTreeMap<SlotName, TeamAndSlot>,
    pub seed_name: SeedName,
    pub minimum_versions: MinimumVersions,
    pub server_options: ServerOptions,
    pub version: PickledVersion,
    #[serde(rename = "datapackage")]
    pub data_package: BTreeMap<String, HashedGameData>,
    pub locations: BTreeMap<SlotId, BTreeMap<LocationId, LocationInfo>>,
    pub spheres: Vec<BTreeMap<SlotId, Vec<LocationId>>>,
    #[serde(flatten)]
    pub rest: BTreeMap<String, PickleValue>,
}

fn deserialize_pickle_slot_data<'de, D>(de: D) -> Result<BTreeMap<SlotId, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let slot_data = <BTreeMap<SlotId, PickleValue>>::deserialize(de)?;
    let slot_data = slot_data
        .into_iter()
        .map(|(slot_id, slot_data)| pickle_to_json(slot_data).map(|slot_data| (slot_id, slot_data)))
        .collect::<Result<BTreeMap<_, _>>>()
        .map_err(de::Error::custom)?;

    Ok(slot_data)
}

fn pickle_to_json(value: PickleValue) -> Result<Value> {
    Ok(match value {
        PickleValue::None => Value::Null,
        PickleValue::Bool(value) => value.into(),
        PickleValue::I64(value) => value.into(),
        PickleValue::Int(value) => i64::try_from(value)
            .map(Value::from)
            .context("failed to convert big int to i64")?,
        PickleValue::F64(value) => value.into(),
        PickleValue::Bytes(value) => value.into(),
        PickleValue::String(value) => value.into(),
        PickleValue::List(values) => values
            .into_iter()
            .map(pickle_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
        PickleValue::Tuple(values) => values
            .into_iter()
            .map(pickle_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
        PickleValue::Set(values) => values
            .into_iter()
            .map(pickle_hashable_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
        PickleValue::FrozenSet(values) => values
            .into_iter()
            .map(pickle_hashable_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
        PickleValue::Dict(values) => values
            .into_iter()
            .flat_map(|kv| pickle_kv_to_json(kv).transpose())
            .collect::<Result<Map<_, _>>>()?
            .into(),
    })
}

fn pickle_kv_to_json(
    (key, value): (HashableValue, PickleValue),
) -> Result<Option<(String, Value)>> {
    let key = match key {
        HashableValue::None => String::new(),
        HashableValue::Bool(value) => value.to_string(),
        HashableValue::I64(value) => value.to_string(),
        HashableValue::Int(value) => value.to_string(),
        HashableValue::F64(value) => value.to_string(),
        HashableValue::Bytes(value) => {
            eprintln!("Found a bytes key: `{}`", value.as_bstr());
            return Ok(None);
        }
        HashableValue::String(value) => value,
        HashableValue::Tuple(value) => {
            eprintln!("Found a tuple key: {value:?}");
            return Ok(None);
        }
        HashableValue::FrozenSet(value) => {
            eprintln!("Found a frozen set key: {value:?}");
            return Ok(None);
        }
    };

    let value = pickle_to_json(value)?;

    Ok(Some((key, value)))
}

fn pickle_hashable_to_json(hashable: HashableValue) -> Result<Value> {
    Ok(match hashable {
        HashableValue::None => Value::Null,
        HashableValue::Bool(value) => value.into(),
        HashableValue::I64(value) => value.into(),
        HashableValue::Int(value) => i64::try_from(value)
            .map(Value::from)
            .context("failed to convert big int to i64")?,
        HashableValue::F64(value) => value.into(),
        HashableValue::Bytes(value) => value.into(),
        HashableValue::String(value) => value.into(),
        HashableValue::Tuple(values) => values
            .into_iter()
            .map(pickle_hashable_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
        HashableValue::FrozenSet(values) => values
            .into_iter()
            .map(pickle_hashable_to_json)
            .collect::<Result<Vec<_>>>()?
            .into(),
    })
}
