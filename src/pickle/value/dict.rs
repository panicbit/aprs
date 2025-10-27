use std::fmt;

use eyre::{Result, bail};
use tracing::warn;

use crate::FnvIndexMap;
use crate::pickle::value::storage::{SameAs, Storage};

use super::Value;

type Map<S> = FnvIndexMap<Value<S>, Value<S>>;

#[derive(Clone)]
pub struct Dict<S: Storage>(S::ReadWrite<Map<S>>);

impl<S> Dict<S>
where
    S: Storage,
{
    pub fn new() -> Self {
        Self(S::new_read_write(Map::default()))
    }

    pub fn insert(&self, key: impl Into<Value<S>>, value: impl Into<Value<S>>) -> Result<()> {
        let key = key.into();
        let value = value.into();

        if !key.is_hashable() {
            bail!("key is not hashable: {key:#?}");
        }

        S::write(&self.0).insert(key, value);

        Ok(())
    }

    pub fn len(&self) -> usize {
        S::read(&self.0).len()
    }

    pub fn read(&self) -> ReadDictGuard<'_, S> {
        ReadDictGuard::new(self)
    }

    pub fn write(&self) -> WriteDictGuard<'_, S> {
        WriteDictGuard::new(self)
    }

    pub fn update(&self, other: &Dict<S>) -> Result<()> {
        if self.0.same_as(&other.0) {
            return Ok(());
        }

        let mut this = self.write();

        for (key, value) in other.read().iter() {
            let key = key.clone();
            let value = value.clone();

            this.insert(key, value)?;
        }

        Ok(())
    }
}

impl<S: Storage> From<Map<S>> for Dict<S> {
    fn from(value: Map<S>) -> Self {
        Self(S::new_read_write(value))
    }
}

impl<S: Storage> PartialEq for Dict<S> {
    fn eq(&self, other: &Self) -> bool {
        let this = self.read();
        let other = other.read();

        if this.len() != other.len() {
            return false;
        }

        for (key, value) in this.iter() {
            if other.get(key) != Some(value) {
                return false;
            }
        }

        for (key, value) in other.iter() {
            if this.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

impl<S: Storage> Default for Dict<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Storage> fmt::Debug for Dict<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.read().iter()).finish()
    }
}

pub struct ReadDictGuard<'a, S: Storage> {
    dict: S::Read<'a, Map<S>>,
}

impl<'a, S: Storage> ReadDictGuard<'a, S> {
    fn new(dict: &'a Dict<S>) -> Self {
        let dict = S::read(&dict.0);

        Self { dict }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value<S>, &Value<S>)> {
        self.dict.iter()
    }

    pub fn len(&self) -> usize {
        self.dict.len()
    }

    pub fn get(&self, key: &Value<S>) -> Option<&Value<S>> {
        self.dict.get(key)
    }
}

pub struct WriteDictGuard<'a, S: Storage> {
    dict: S::Write<'a, Map<S>>,
}

impl<'a, S: Storage> WriteDictGuard<'a, S> {
    fn new(dict: &'a Dict<S>) -> Self {
        let dict = S::write(&dict.0);

        Self { dict }
    }

    pub fn insert(&mut self, key: Value<S>, value: Value<S>) -> Result<()> {
        if !key.is_hashable() {
            bail!("key is not hashable: {key:#?}");
        }

        self.dict.insert(key, value);

        Ok(())
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = (Value<S>, Value<S>)>) {
        let items = items.into_iter().filter(|(key, _)| {
            if !key.is_hashable() {
                warn!("dict key is not hashable: {key:#?}");

                return false;
            }

            true
        });

        self.dict.extend(items);
    }
}
