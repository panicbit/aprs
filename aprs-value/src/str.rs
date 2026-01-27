use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

use eyre::{Error, Result, bail};
use serde::{Deserialize, Serialize};

use crate::Value;
use crate::storage::Storage;

#[derive(PartialEq, Eq, Clone)]
pub struct Str<S: Storage>(S::ReadOnly<String>);

impl<S: Storage> Str<S> {
    pub fn new() -> Self {
        Self(S::new_read_only(String::new()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<S: Storage> Default for Str<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Storage> Hash for Str<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<S: Storage> From<String> for Str<S> {
    fn from(value: String) -> Self {
        Self(S::new_read_only(value))
    }
}

impl<S: Storage> From<&'_ str> for Str<S> {
    fn from(value: &'_ str) -> Self {
        Self(S::new_read_only(String::from(value)))
    }
}

impl<S: Storage> TryFrom<Value<S>> for Str<S> {
    type Error = Error;

    fn try_from(value: Value<S>) -> Result<Self> {
        if let Value::Str(str) = value {
            Ok(str)
        } else {
            bail!("{} is not a Str", value.type_name())
        }
    }
}

impl<S: Storage> Deref for Str<S> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<S: Storage> fmt::Debug for Str<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<S: Storage> Borrow<str> for Str<S> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'de, S: Storage> Deserialize<'de> for Str<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        let str = Self::from(str);

        Ok(str)
    }
}

impl<ST: Storage> Serialize for Str<ST> {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str(&self.0)
    }
}
