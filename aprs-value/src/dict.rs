use std::fmt;
use std::sync::Arc;

use eyre::{Result, bail};
use parking_lot::RwLockWriteGuard;
use parking_lot::{RwLock, RwLockReadGuard};
use tracing::warn;

use crate::FnvIndexMap;
use crate::Int;

use super::Value;

#[derive(Clone)]
pub struct Dict(Arc<RwLock<Inner>>);

#[derive(PartialEq, Eq)]
struct Inner {
    value_dict: FnvIndexMap<Value, Value>,
    int_dict: FnvIndexMap<i64, Value>,
}

impl Dict {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Inner {
            value_dict: <_>::default(),
            int_dict: <_>::default(),
        })))
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub fn read(&self) -> ReadDictGuard<'_> {
        ReadDictGuard::new(self)
    }

    pub fn write(&self) -> WriteDictGuard<'_> {
        WriteDictGuard::new(self)
    }

    pub fn update(&self, other: &Dict) -> Result<()> {
        if Arc::ptr_eq(&self.0, &other.0) {
            return Ok(());
        }

        let mut this = self.write();

        for (key, value) in other.read().iter() {
            let key = Value::from(key);
            let value = value.clone();

            this.insert(key, value)?;
        }

        Ok(())
    }
}

impl Inner {
    fn len(&self) -> usize {
        self.value_dict.len() + self.int_dict.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, key: &Value) -> Option<&Value> {
        match &key {
            Value::Int(Int::I64(key)) => self.int_dict.get(key),
            _ => self.value_dict.get(key),
        }
    }

    fn iter(&self) -> impl Iterator<Item = (Key<'_>, &Value)> {
        let value_iter = self
            .value_dict
            .iter()
            .map(|(key, value)| (Key::Value(key), value));
        let int_iter = self
            .int_dict
            .iter()
            .map(|(key, value)| (Key::Int64(*key), value));

        value_iter.chain(int_iter)
    }

    fn insert(&mut self, key: impl Into<Value>, value: impl Into<Value>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("dict key is not hashable: {key:#?}");
        }

        let value = value.into();

        match key {
            Value::Int(Int::I64(key)) => self.int_dict.insert(key, value),
            _ => self.value_dict.insert(key, value),
        };

        Ok(())
    }

    fn extend(&mut self, items: impl IntoIterator<Item = (Value, Value)>) {
        for (key, value) in items.into_iter() {
            if let Err(err) = self.insert(key, value) {
                warn!("{err}");
            }
        }
    }
}

impl PartialEq for Dict {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }

        let this = self.read();
        let other = other.read();

        *this.inner == *other.inner
    }
}

impl Default for Dict {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Dict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.read().iter()).finish()
    }
}

pub struct ReadDictGuard<'a> {
    inner: RwLockReadGuard<'a, Inner>,
}

impl<'a> ReadDictGuard<'a> {
    fn new(dict: &'a Dict) -> Self {
        let inner = dict.0.read();

        Self { inner }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Key<'_>, &Value)> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.inner.get(key)
    }
}

pub struct WriteDictGuard<'a> {
    inner: RwLockWriteGuard<'a, Inner>,
}

impl<'a> WriteDictGuard<'a> {
    fn new(dict: &'a Dict) -> Self {
        let inner = dict.0.write();

        Self { inner }
    }

    pub fn insert(&mut self, key: impl Into<Value>, value: impl Into<Value>) -> Result<()> {
        self.inner.insert(key, value)
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = (Value, Value)>) {
        self.inner.extend(items);
    }
}

#[derive(Clone, Debug)]
pub enum Key<'a> {
    Value(&'a Value),
    Int64(i64),
}

impl From<Key<'_>> for Value {
    fn from(key: Key<'_>) -> Self {
        match key {
            Key::Value(value) => value.clone(),
            Key::Int64(value) => Value::Int(value.into()),
        }
    }
}
