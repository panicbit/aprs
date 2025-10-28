use std::fmt;

use eyre::{Result, bail};
use tracing::warn;

use crate::FnvIndexSet;
use crate::pickle::value::Number;
use crate::pickle::value::number::N;
use crate::pickle::value::storage::{SameAs, Storage};

use super::Value;

#[derive(Clone)]
pub struct Set<S: Storage>(S::ReadWrite<Inner<S>>);

#[derive(PartialEq)]
struct Inner<S: Storage> {
    value_set: FnvIndexSet<Value<S>>,
    int_set: FnvIndexSet<i64>,
}

impl<S: Storage> Set<S> {
    pub fn new() -> Self {
        Self(S::new_read_write(Inner {
            value_set: <_>::default(),
            int_set: <_>::default(),
        }))
    }

    pub fn read(&self) -> ReadSetGuard<S> {
        ReadSetGuard::new(self)
    }

    pub fn write(&self) -> WriteSetGuard<S> {
        WriteSetGuard::new(self)
    }
}

impl<S: Storage> Inner<S> {
    fn insert(&mut self, key: impl Into<Value<S>>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("set key is not hashable: {key:?}");
        }

        match key {
            Value::Number(Number(N::I64(key))) => {
                self.int_set.insert(key);
            }
            _ => {
                self.value_set.insert(key);
            }
        };

        Ok(())
    }

    fn iter(&self) -> impl Iterator<Item = Item<S>> {
        let value_set = self.value_set.iter().map(Item::Value);
        let int_set = self.int_set.iter().copied().map(Item::Int64);

        value_set.chain(int_set)
    }

    fn len(&self) -> usize {
        self.value_set.len()
    }

    fn extend(&mut self, items: impl IntoIterator<Item = Value<S>>) {
        for item in items.into_iter() {
            if let Err(err) = self.insert(item) {
                warn!("{err}");
            }
        }
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

        *this.inner == *other.inner
    }
}

impl<S: Storage> fmt::Debug for Set<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.read().iter()).finish()
    }
}

pub struct ReadSetGuard<'a, S: Storage> {
    inner: S::Read<'a, Inner<S>>,
}

impl<'a, S: Storage> ReadSetGuard<'a, S> {
    fn new(set: &'a Set<S>) -> Self {
        let set = S::read(&set.0);

        Self { inner: set }
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<S>> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct WriteSetGuard<'a, S: Storage> {
    inner: S::Write<'a, Inner<S>>,
}

impl<'a, S: Storage> WriteSetGuard<'a, S> {
    fn new(set: &'a Set<S>) -> Self {
        let inner = S::write(&set.0);

        Self { inner }
    }

    pub fn iter(&self) -> impl Iterator<Item = Item<S>> {
        self.inner.iter()
    }

    pub fn insert(&mut self, key: Value<S>) -> Result<()> {
        self.inner.insert(key)
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = Value<S>>) {
        self.inner.extend(items)
    }
}

#[derive(Clone, Debug)]
pub enum Item<'a, S: Storage> {
    Value(&'a Value<S>),
    Int64(i64),
}

impl<S: Storage> From<Item<'_, S>> for Value<S> {
    fn from(key: Item<'_, S>) -> Self {
        match key {
            Item::Value(value) => value.clone(),
            Item::Int64(value) => Value::Number(value.into()),
        }
    }
}
