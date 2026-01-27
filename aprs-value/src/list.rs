use std::fmt;

use eyre::{ContextCompat, Result};

use crate::storage::Storage;

use super::Value;

#[derive(Clone)]
pub struct List<S: Storage>(S::ReadWrite<Vec<Value<S>>>);

impl<S: Storage> List<S> {
    pub fn new() -> Self {
        Self(S::new_read_write(Vec::new()))
    }

    pub fn iter(&self) -> Iter<S> {
        self.into_iter()
    }

    pub fn read(&self) -> ReadListGuard<'_, S> {
        ReadListGuard::new(self)
    }

    pub fn len(&self) -> usize {
        S::read(&self.0).len()
    }

    pub fn is_empty(&self) -> bool {
        S::read(&self.0).is_empty()
    }

    pub fn last(&self) -> Option<Value<S>> {
        S::read(&self.0).last().cloned()
    }

    pub fn push(&self, value: Value<S>) {
        S::write(&self.0).push(value);
    }

    pub fn pop(&self) -> Option<Value<S>> {
        S::write(&self.0).pop()
    }

    pub fn get<I>(&self, index: I) -> I::Output<Option<Value<S>>>
    where
        I: ListIndex,
    {
        index.map(|index| S::read(&self.0).get(index).cloned())
    }

    pub fn remove<I>(&self, index: I) -> I::Output<Option<Value<S>>>
    where
        I: ListIndex,
    {
        index.map(|index| {
            let mut list = S::write(&self.0);

            if index >= list.len() {
                return None;
            }

            Some(list.remove(index))
        })
    }

    pub fn extend(&self, values: Vec<Value<S>>) {
        S::write(&self.0).extend(values);
    }

    pub fn append_list(&self, list: &List<S>) -> Result<()> {
        let list = S::read(&list.0).clone();

        S::write(&self.0).extend(list);

        Ok(())
    }
}

impl<S: Storage> Default for List<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Storage> From<Vec<Value<S>>> for List<S> {
    fn from(values: Vec<Value<S>>) -> Self {
        Self(S::new_read_write(values))
    }
}

impl<S: Storage> IntoIterator for List<S> {
    type Item = Value<S>;
    type IntoIter = Iter<S>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl<S: Storage> IntoIterator for &List<S> {
    type Item = Value<S>;
    type IntoIter = Iter<S>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
    }
}

pub struct ReadListGuard<'a, S: Storage> {
    list: S::Read<'a, Vec<Value<S>>>,
}

impl<'a, S: Storage> ReadListGuard<'a, S> {
    fn new(list: &'a List<S>) -> Self {
        let list = S::read(&list.0);

        Self { list }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value<S>> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

impl<'a, S: Storage> IntoIterator for &'a ReadListGuard<'a, S> {
    type Item = &'a Value<S>;
    type IntoIter = std::slice::Iter<'a, Value<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.iter()
    }
}

pub struct Iter<S: Storage> {
    list: List<S>,
    index: usize,
    max_len: usize,
}

impl<S: Storage> Iter<S> {
    fn new(list: List<S>) -> Self {
        Iter {
            max_len: list.len(),
            list,
            index: 0,
        }
    }
}

impl<S: Storage> Iterator for Iter<S> {
    type Item = Value<S>;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = S::read(&self.list.0);

        // This prevents appending a list to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let value = vec.get(self.index)?.clone();

        self.index += 1;

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max_len - self.index.min(self.max_len)))
    }
}

impl<V, S: Storage> FromIterator<V> for List<S>
where
    V: Into<Value<S>>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        // TODO: consider iterator size_hint
        let list = List::new();

        for value in iter {
            let value = value.into();

            list.push(value);
        }

        list
    }
}

impl<S: Storage> fmt::Debug for List<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<S: Storage> PartialEq for List<S> {
    fn eq(&self, other: &Self) -> bool {
        self.iter().zip(other).all(|(v1, v2)| v1 == v2)
    }
}

pub trait ListIndex {
    type Output<T>;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T;
}

impl ListIndex for usize {
    type Output<T> = T;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T,
    {
        f(self)
    }
}

impl<S: Storage> ListIndex for Value<S> {
    type Output<T> = Result<T>;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T,
    {
        let index = self.to_usize().context("invalid list index")?;

        Ok(f(index))
    }
}

impl<S: Storage> ListIndex for &Value<S> {
    type Output<T> = Result<T>;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T,
    {
        let index = self.to_usize().context("invalid list index")?;

        Ok(f(index))
    }
}
