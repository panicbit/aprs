use std::fmt;
use std::sync::Arc;

use eyre::{Result, bail};
use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;
use parking_lot::RwLockWriteGuard;
use tracing::warn;

use crate::FnvIndexSet;
use crate::Int;

use super::Value;

#[derive(Clone)]
pub struct Set(Arc<RwLock<Inner>>);

#[derive(PartialEq)]
struct Inner {
    value_set: FnvIndexSet<Value>,
    int_set: FnvIndexSet<i64>,
}

impl Set {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Inner {
            value_set: <_>::default(),
            int_set: <_>::default(),
        })))
    }

    pub fn read(&self) -> ReadSetGuard<'_> {
        ReadSetGuard::new(self)
    }

    pub fn write(&self) -> WriteSetGuard<'_> {
        WriteSetGuard::new(self)
    }
}

impl Inner {
    fn insert(&mut self, key: impl Into<Value>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("set key is not hashable: {key:?}");
        }

        match key {
            Value::Int(Int::I64(key)) => {
                self.int_set.insert(key);
            }
            _ => {
                self.value_set.insert(key);
            }
        };

        Ok(())
    }

    fn iter(&self) -> impl Iterator<Item = Item<'_>> {
        let value_set = self.value_set.iter().map(Item::Value);
        let int_set = self.int_set.iter().copied().map(Item::Int64);

        value_set.chain(int_set)
    }

    fn len(&self) -> usize {
        self.value_set.len()
    }

    fn extend(&mut self, items: impl IntoIterator<Item = Value>) {
        for item in items.into_iter() {
            if let Err(err) = self.insert(item) {
                warn!("{err}");
            }
        }
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0, &other.0) {
            return true;
        }

        let this = self.read();
        let other = other.read();

        *this.inner == *other.inner
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.read().iter()).finish()
    }
}

pub struct ReadSetGuard<'a> {
    inner: RwLockReadGuard<'a, Inner>,
}

impl<'a> ReadSetGuard<'a> {
    fn new(set: &'a Set) -> Self {
        let set = set.0.read();

        Self { inner: set }
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<'_>> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct WriteSetGuard<'a> {
    inner: RwLockWriteGuard<'a, Inner>,
}

impl<'a> WriteSetGuard<'a> {
    fn new(set: &'a Set) -> Self {
        let inner = set.0.write();

        Self { inner }
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<'_>> {
        self.inner.iter()
    }

    pub fn insert(&mut self, key: Value) -> Result<()> {
        self.inner.insert(key)
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = Value>) {
        self.inner.extend(items)
    }
}

#[derive(Clone, Debug)]
pub enum Item<'a> {
    Value(&'a Value),
    Int64(i64),
}

impl From<Item<'_>> for Value {
    fn from(key: Item<'_>) -> Self {
        match key {
            Item::Value(value) => value.clone(),
            Item::Int64(value) => Value::Int(value.into()),
        }
    }
}
