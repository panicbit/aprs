use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

use anyhow::{Error, Result};
use dumpster::Trace;
use dumpster::sync::Gc;

use crate::pickle::value::{Id, Value};

#[derive(Trace, PartialEq, Clone)]
pub struct Str(Gc<String>);

impl Str {
    pub fn new() -> Self {
        Self(Gc::new(String::new()))
    }

    pub fn id(&self) -> Id {
        Gc::as_ptr(&self.0).into()
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
        Self(Gc::new(value))
    }
}

impl From<&'_ str> for Str {
    fn from(value: &'_ str) -> Self {
        Self(Gc::new(String::from(value)))
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
