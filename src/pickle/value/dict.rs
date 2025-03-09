use std::cmp;
use std::hash::{Hash, Hasher};

use anyhow::{Result, bail};
use dumpster::Trace;
use dumpster::sync::Gc;
use indexmap::IndexMap;
use parking_lot::RwLock;

use super::Value;

#[derive(Default)]
pub struct Dict(RwLock<IndexMap<Element, Element>>);

impl Dict {
    pub fn new() -> Gc<Self> {
        Gc::new(Self(RwLock::new(IndexMap::new())))
    }

    pub fn insert(&self, key: Value, value: Value) -> Result<()> {
        if !key.is_hashable() {
            bail!("key is not hashable");
        }

        self.0.write().insert(Element(key), Element(value));
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn get(&self, key: Value) -> Option<Value> {
        self.0
            .read()
            .get(&Element(key.clone()))
            .map(|Element(value)| value)
            .cloned()
    }

    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }

    pub fn values(&self) -> Values<'_> {
        Values {
            dict: self,
            index: 0,
            max_len: self.len(),
        }
    }
}

impl PartialEq for Dict {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in self {
            if other.get(key) != Some(value) {
                return false;
            }
        }

        for (key, value) in other {
            if self.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

unsafe impl Trace for Dict {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        for (key, value) in self {
            key.accept(visitor)?;
            value.accept(visitor)?;
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a Dict {
    type Item = (Value, Value);
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            dict: self,
            index: 0,
            max_len: self.len(),
        }
    }
}

pub struct Iter<'a> {
    dict: &'a Dict,
    index: usize,
    max_len: usize,
}

impl Iterator for Iter<'_> {
    type Item = (Value, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.dict.0.read();

        // This prevents appending a dict to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let (key, value) = vec.get_index(self.index)?;
        let value = (key.0.clone(), value.0.clone());

        self.index += 1;

        Some(value)
    }
}

pub struct Values<'a> {
    dict: &'a Dict,
    index: usize,
    max_len: usize,
}

impl Iterator for Values<'_> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.dict.0.read();

        // This prevents appending a dict to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let (_, value) = vec.get_index(self.index)?;
        let value = value.0.clone();

        self.index += 1;

        Some(value)
    }
}

struct Element(Value);

impl Hash for Element {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0
            .hash(state)
            .unwrap_or_else(|err| eprintln!("hash of unhashable value: {err}"));
    }
}

impl cmp::PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// This is a lie. All bets are off.
impl cmp::Eq for Element {}
