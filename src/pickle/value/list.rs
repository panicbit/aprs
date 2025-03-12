use std::fmt;

use anyhow::{Result, bail};
use dumpster::Trace;

use crate::pickle::value::Id;
use crate::pickle::value::rw_gc::RwGc;

use super::Value;

#[derive(Clone, Trace)]
pub struct List(RwGc<Vec<Value>>);

impl List {
    pub fn new() -> Self {
        Self(RwGc::new(Vec::new()))
    }

    pub fn id(&self) -> Id {
        self.0.id()
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
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

    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.read().get(index).cloned()
    }

    pub fn extend(&self, values: Value) -> Result<()> {
        match values {
            Value::List(values) => self.append_list(&values),
            _ => bail!("can't extend List with {}", values.type_name()),
        }
    }

    fn append_list(&self, list: &List) -> Result<()> {
        for value in list {
            self.push(value);
        }

        Ok(())
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
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
        (0, Some(self.max_len - self.index.max(self.max_len)))
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
