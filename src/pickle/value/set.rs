use std::fmt;

use eyre::{Result, bail};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::FnvIndexSet;
use crate::pickle::value::rw_arc::RwArc;

use super::Value;

#[derive(Clone)]
pub struct Set(RwArc<FnvIndexSet<Value>>);

impl Set {
    pub fn new() -> Self {
        Self(RwArc::new(FnvIndexSet::default()))
    }

    pub fn insert(&self, key: impl Into<Value>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("key is not hashable: {key:?}");
        }

        self.0.write().insert(key);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0.read().get(key).cloned()
    }

    pub fn contains(&self, key: Value) -> bool {
        self.0.read().contains(&key)
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn read(&self) -> ReadSetGuard {
        ReadSetGuard::new(self)
    }

    pub fn write(&self) -> WriteSetGuard {
        WriteSetGuard::new(self)
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Set {
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

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl IntoIterator for Set {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl IntoIterator for &Set {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
    }
}

pub struct ReadSetGuard<'a> {
    set: RwLockReadGuard<'a, FnvIndexSet<Value>>,
}

impl<'a> ReadSetGuard<'a> {
    fn new(set: &'a Set) -> Self {
        let set = set.0.read();

        Self { set }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.set.iter()
    }
}

pub struct WriteSetGuard<'a> {
    set: RwLockWriteGuard<'a, FnvIndexSet<Value>>,
}

impl<'a> WriteSetGuard<'a> {
    fn new(set: &'a Set) -> Self {
        let set = set.0.write();

        Self { set }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.set.iter()
    }

    pub fn insert(&mut self, key: Value) -> Result<()> {
        if !key.is_hashable() {
            bail!("key is not hashable: {key:?}");
        }

        self.set.insert(key);

        Ok(())
    }
}

pub struct Iter {
    set: Set,
    index: usize,
    max_len: usize,
}

impl Iter {
    fn new(set: Set) -> Self {
        Self {
            max_len: set.len(),
            set,
            index: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.set.0.read();

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
