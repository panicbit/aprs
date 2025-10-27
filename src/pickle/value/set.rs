use std::fmt;

use eyre::{Result, bail};
use tracing::warn;

use crate::FnvIndexSet;
use crate::pickle::value::storage::Storage;

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

    pub fn len(&self) -> usize {
        S::read(&self.0).len()
    }

    pub fn get(&self, key: &Value<S>) -> Option<Value<S>> {
        S::read(&self.0).get(key).cloned()
    }

    pub fn contains(&self, key: Value<S>) -> bool {
        S::read(&self.0).contains(&key)
    }

    pub fn iter(&self) -> Iter<S> {
        self.into_iter()
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
        if self.len() != other.len() {
            return false;
        }

        for key in self {
            if !other.contains(key) {
                return false;
            }
        }

        for key in other {
            if !self.contains(key) {
                return false;
            }
        }

        true
    }
}

impl<S: Storage> fmt::Debug for Set<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<S: Storage> IntoIterator for Set<S> {
    type Item = Value<S>;
    type IntoIter = Iter<S>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<S: Storage> IntoIterator for &Set<S> {
    type Item = Value<S>;
    type IntoIter = Iter<S>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
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

pub struct Iter<S: Storage> {
    set: Set<S>,
    index: usize,
    max_len: usize,
}

impl<S: Storage> Iter<S> {
    fn new(set: Set<S>) -> Self {
        Self {
            max_len: set.len(),
            set,
            index: 0,
        }
    }
}

impl<S: Storage> Iterator for Iter<S> {
    type Item = Value<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = S::read(&self.set.0);

        // This prevents appending a set to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let value = vec.get_index(self.index)?.clone();

        self.index += 1;

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max_len - self.index.min(self.max_len)))
    }
}
