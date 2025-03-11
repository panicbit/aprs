use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

use anyhow::{Result, bail};
use dumpster::Trace;

use crate::FnvIndexMap;
use crate::pickle::value::Id;
use crate::pickle::value::rw_gc::RwGc;
use crate::pickle::value::traced::Traced;

use super::Value;

#[derive(Clone, Trace)]
pub struct Dict(RwGc<Traced<FnvIndexMap<Element, Element>>>);

impl Dict {
    pub fn new() -> Self {
        Self(RwGc::new(Traced(FnvIndexMap::default())))
    }

    pub fn id(&self) -> Id {
        self.0.id()
    }

    pub fn insert(&self, key: impl Into<Value>, value: impl Into<Value>) -> Result<()> {
        let key = key.into();
        let value = value.into();

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
            .get(&Element(key))
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

impl Default for Dict {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Dict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self).finish()
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

        let (Element(key), Element(value)) = vec.get_index(self.index)?;
        let value = (key.clone(), value.clone());

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

#[derive(Trace)]
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
