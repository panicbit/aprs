use std::borrow::Borrow;
use std::{fmt, ops};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ConnectName(pub String);

impl Default for ConnectName {
    fn default() -> Self {
        Self::new()
    }
}

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
