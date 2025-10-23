use std::fmt;
use std::hash::Hash;
use std::ops::Deref;
use std::{borrow::Borrow, sync::Arc};

use eyre::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::pickle::value::{Id, Value};

#[derive(PartialEq, Eq, Clone)]
pub struct Str(Arc<String>);

impl Str {
    pub fn new() -> Self {
        Self(Arc::new(String::new()))
    }

    pub fn id(&self) -> Id {
        Arc::as_ptr(&self.0).into()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Str {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Str {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<String> for Str {
    fn from(value: String) -> Self {
        Self(Arc::new(value))
    }
}

impl From<&'_ str> for Str {
    fn from(value: &'_ str) -> Self {
        Self(Arc::new(String::from(value)))
    }
}

impl TryFrom<Value> for Str {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        value.as_str()
    }
}

impl Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Borrow<str> for Str {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'de> Deserialize<'de> for Str {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = String::deserialize(deserializer)?;
        let str = Self::from(str);

        Ok(str)
    }
}

impl Serialize for Str {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str(&self.0)
    }
}
