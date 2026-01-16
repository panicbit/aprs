use std::fmt;

use eyre::{Result, bail};
use tracing::warn;

use crate::FnvIndexMap;
use crate::pickle::value::Number;
use crate::pickle::value::number::N;
use crate::pickle::value::storage::{SameAs, Storage};

use super::Value;

#[derive(Clone)]
pub struct Dict<S: Storage>(S::ReadWrite<Inner<S>>);

#[derive(PartialEq, Eq)]
struct Inner<S: Storage> {
    value_dict: FnvIndexMap<Value<S>, Value<S>>,
    int_dict: FnvIndexMap<i64, Value<S>>,
}

impl<S> Dict<S>
where
    S: Storage,
{
    pub fn new() -> Self {
        Self(S::new_read_write(Inner {
            value_dict: <_>::default(),
            int_dict: <_>::default(),
        }))
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
            let key = Value::from(key);
            let value = value.clone();

            this.insert(key, value)?;
        }

        Ok(())
    }
}

impl<S: Storage> Inner<S> {
    fn len(&self) -> usize {
        self.value_dict.len() + self.int_dict.len()
    }

    fn get(&self, key: &Value<S>) -> Option<&Value<S>> {
        match &key {
            Value::Number(Number(N::I64(key))) => self.int_dict.get(key),
            _ => self.value_dict.get(key),
        }
    }

    fn iter(&self) -> impl Iterator<Item = (Key<'_, S>, &Value<S>)> {
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

    fn insert(&mut self, key: impl Into<Value<S>>, value: impl Into<Value<S>>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("dict key is not hashable: {key:#?}");
        }

        let value = value.into();

        match key {
            Value::Number(Number(N::I64(key))) => self.int_dict.insert(key, value),
            _ => self.value_dict.insert(key, value),
        };

        Ok(())
    }

    fn extend(&mut self, items: impl IntoIterator<Item = (Value<S>, Value<S>)>) {
        for (key, value) in items.into_iter() {
            if let Err(err) = self.insert(key, value) {
                warn!("{err}");
            }
        }
    }
}

impl<S: Storage> PartialEq for Dict<S> {
    fn eq(&self, other: &Self) -> bool {
        if self.0.same_as(&other.0) {
            return true;
        }

        let this = self.read();
        let other = other.read();

        *this.inner == *other.inner
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
    inner: S::Read<'a, Inner<S>>,
}

impl<'a, S: Storage> ReadDictGuard<'a, S> {
    fn new(dict: &'a Dict<S>) -> Self {
        let inner = S::read(&dict.0);

        Self { inner }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Key<'_, S>, &Value<S>)> {
        self.inner.iter()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, key: &Value<S>) -> Option<&Value<S>> {
        self.inner.get(key)
    }
}

pub struct WriteDictGuard<'a, S: Storage> {
    inner: S::Write<'a, Inner<S>>,
}

impl<'a, S: Storage> WriteDictGuard<'a, S> {
    fn new(dict: &'a Dict<S>) -> Self {
        let inner = S::write(&dict.0);

        Self { inner }
    }

    pub fn insert(&mut self, key: impl Into<Value<S>>, value: impl Into<Value<S>>) -> Result<()> {
        self.inner.insert(key, value)
    }

    pub fn extend(&mut self, items: impl IntoIterator<Item = (Value<S>, Value<S>)>) {
        self.inner.extend(items);
    }
}

#[derive(Clone, Debug)]
pub enum Key<'a, S: Storage> {
    Value(&'a Value<S>),
    Int64(i64),
}

impl<S: Storage> From<Key<'_, S>> for Value<S> {
    fn from(key: Key<'_, S>) -> Self {
        match key {
            Key::Value(value) => value.clone(),
            Key::Int64(value) => Value::Number(value.into()),
        }
    }
}
