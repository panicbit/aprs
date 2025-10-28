use std::fmt;

use eyre::{Result, bail};
use tracing::warn;

use crate::FnvIndexSet;
use crate::pickle::value::storage::{SameAs, Storage};

use super::Value;

#[derive(Clone)]
pub struct Set<S: Storage>(S::ReadWrite<FnvIndexSet<Value<S>>>);

impl<S: Storage> Set<S> {
    pub fn new() -> Self {
        Self(S::new_read_write(FnvIndexSet::default()))
    }

    pub fn insert(&self, key: impl Into<Value<S>>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("key is not hashable: {key:?}");
        }

        S::write(&self.0).insert(key);

        Ok(())
    }

    pub fn get(&self, key: &Value<S>) -> Option<Value<S>> {
        S::read(&self.0).get(key).cloned()
    }

    pub fn contains(&self, key: Value<S>) -> bool {
        S::read(&self.0).contains(&key)
    }

    pub fn read(&self) -> ReadSetGuard<S> {
        ReadSetGuard::new(self)
    }

    pub fn write(&self) -> WriteSetGuard<S> {
        WriteSetGuard::new(self)
    }
}

impl<S: Storage> Default for Set<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Storage> PartialEq for Set<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.same_as(&other.0) {
            return true;
        }

        let this = self.read();
        let other = other.read();

        *this.set == *other.set
    }
}

impl<S: Storage> fmt::Debug for Set<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.read().iter()).finish()
    }
}

pub struct ReadSetGuard<'a, S: Storage> {
    set: S::Read<'a, FnvIndexSet<Value<S>>>,
}

impl<'a, S: Storage> ReadSetGuard<'a, S> {
    fn new(set: &'a Set<S>) -> Self {
        let set = S::read(&set.0);

        Self { set }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value<S>> {
        self.set.iter()
    }

    pub fn len(&self) -> usize {
        self.set.len()
    }
}

pub struct WriteSetGuard<'a, S: Storage> {
    set: S::Write<'a, FnvIndexSet<Value<S>>>,
}

impl<'a, S: Storage> WriteSetGuard<'a, S> {
    fn new(set: &'a Set<S>) -> Self {
        let set = S::write(&set.0);

        Self { set }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value<S>> {
        self.set.iter()
    }

    pub fn insert(&mut self, key: Value<S>) -> Result<()> {
        if !key.is_hashable() {
            bail!("key is not hashable: {key:?}");
        }

        self.set.insert(key);

        Ok(())
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = Value<S>>) {
        let items = items.into_iter().filter(|item| {
            if !item.is_hashable() {
                warn!("set item is not hashable: {item:#?}");

                return false;
            }

            true
        });

        self.set.extend(items);
    }
}
