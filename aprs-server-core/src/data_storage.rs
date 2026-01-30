use std::borrow::Borrow;
use std::hash::Hash;

use aprs_proto::client::{Set, SetOperation};
use aprs_value::Value;
use eyre::{Result, bail};
use fnv::FnvHashMap;
use serde::{Deserialize, Serialize};

// TODO: find instances of HashMap/HashSet that don't use FxHasher
#[derive(Deserialize, Serialize)]
pub struct DataStorage(FnvHashMap<String, Value>);

impl DataStorage {
    pub fn new() -> Self {
        Self(FnvHashMap::default())
    }

    pub fn get_raw<K>(&self, key: &K) -> Option<&Value>
    where
        K: Hash + Eq + ?Sized,
        String: Borrow<K>,
    {
        self.0.get(key)
    }

    pub fn set_raw(&mut self, key: String, value: Value) {
        self.0.insert(key, value);
    }

    /// Returns `(original_value, new_value)`
    pub fn set(&mut self, set: &Set<Value>) -> Result<(Value, Value)> {
        let Set {
            key,
            default,
            want_reply: _,
            operations,
        } = set;

        let original_value = self.get_raw(key).unwrap_or(default).clone();
        let mut value = original_value.clone();

        fn handle_op(current: Value, operation: &SetOperation<Value>) -> Result<Value> {
            Ok(match operation {
                SetOperation::Default => current,
                SetOperation::Replace(value) => value.clone(),
                // TODO: implement remaining set ops
                SetOperation::Add(value) => current.add(value)?,
                SetOperation::Mul(value) => current.mul(value)?,
                // SetOperation::Pow(value) => current.pow(value),
                // SetOperation::Mod(value) => current.r#mod(value),
                SetOperation::Floor => current.floor()?,
                SetOperation::Ceil => current.ceil()?,
                // SetOperation::Max(value) => current.max(value),
                // SetOperation::Min(value) => current.min(value),
                SetOperation::And(value) => current.and(value)?,
                SetOperation::Or(value) => current.or(value)?,
                // SetOperation::Xor(value) => current.xor(value),
                // SetOperation::LeftShift(value) => current.left_shift(value),
                // SetOperation::RightShift(value) => current.right_shift(value),
                // SetOperation::Remove(value) => current.remove(value),
                SetOperation::Pop(value) => current.pop(value).map(|_| current)?,
                SetOperation::Update(value) => current.update(value).map(|_| current)?,
                _ => bail!("TODO: implement SetOperation: {operation:?}"),
            })
        }

        for operation in operations {
            value = match handle_op(value, operation) {
                Ok(value) => value,
                Err(err) => {
                    bail!("op err: {err:?}");
                }
            }
        }

        self.set_raw(key.clone(), value.clone());

        Ok((original_value, value))
    }
}

impl Default for DataStorage {
    fn default() -> Self {
        Self::new()
    }
}
