use std::error::Error as StdError;
use std::fmt;

use eyre::anyhow;
use serde::{de, ser};

pub struct SerdeError(eyre::Error);

impl SerdeError {
    pub fn msg(msg: impl fmt::Display) -> SerdeError {
        Self(anyhow!("{msg}"))
    }
}

impl StdError for SerdeError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl de::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self(anyhow!("{msg}"))
    }
}

impl ser::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self(anyhow!("{msg}"))
    }
}

impl fmt::Debug for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
