use std::fmt;
use std::sync::Arc;

use eyre::{ContextCompat, Result};
use parking_lot::{RwLock, RwLockReadGuard};

use super::Value;

#[derive(Clone)]
pub struct List(Arc<RwLock<Vec<Value>>>);

impl List {
    pub fn new() -> Self {
        Self(Arc::new(RwLock::new(Vec::new())))
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn read(&self) -> ReadListGuard<'_> {
        ReadListGuard::new(self)
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.read().is_empty()
    }

    pub fn last(&self) -> Option<Value> {
        self.0.read().last().cloned()
    }

    pub fn push(&self, value: Value) {
        self.0.write().push(value);
    }

    pub fn pop(&self) -> Option<Value> {
        self.0.write().pop()
    }

    pub fn get<I>(&self, index: I) -> I::Output<Option<Value>>
    where
        I: ListIndex,
    {
        index.map(|index| self.0.read().get(index).cloned())
    }

    pub fn remove<I>(&self, index: I) -> I::Output<Option<Value>>
    where
        I: ListIndex,
    {
        index.map(|index| {
            let mut list = self.0.write();

            if index >= list.len() {
                return None;
            }

            Some(list.remove(index))
        })
    }

    pub fn extend(&self, values: Vec<Value>) {
        self.0.write().extend(values);
    }

    pub fn append_list(&self, list: &List) -> Result<()> {
        let list = list.0.read().clone();

        self.0.write().extend(list);

        Ok(())
    }

    pub fn concat(&self, other: &List) -> List {
        self.iter().chain(other).collect()
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Value>> for List {
    fn from(values: Vec<Value>) -> Self {
        Self(Arc::new(RwLock::new(values)))
    }
}

impl IntoIterator for List {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl IntoIterator for &List {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
    }
}

pub struct ReadListGuard<'a> {
    list: RwLockReadGuard<'a, Vec<Value>>,
}

impl<'a> ReadListGuard<'a> {
    fn new(list: &'a List) -> Self {
        let list = list.0.read();

        Self { list }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value> {
        self.list.iter()
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }
}

impl<'a> IntoIterator for &'a ReadListGuard<'a> {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.list.iter()
    }
}

pub struct Iter {
    list: List,
    index: usize,
    max_len: usize,
}

impl Iter {
    fn new(list: List) -> Self {
        Iter {
            max_len: list.len(),
            list,
            index: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.list.0.read();

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

impl<V> FromIterator<V> for List
where
    V: Into<Value>,
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

impl fmt::Debug for List {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl PartialEq for List {
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

impl ListIndex for Value {
    type Output<T> = Result<T>;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T,
    {
        let index = self.to_usize().context("invalid list index")?;

        Ok(f(index))
    }
}

impl ListIndex for &Value {
    type Output<T> = Result<T>;

    fn map<F, T>(self, f: F) -> Self::Output<T>
    where
        F: FnOnce(usize) -> T,
    {
        let index = self.to_usize().context("invalid list index")?;

        Ok(f(index))
    }
}
