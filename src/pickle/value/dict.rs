use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

use eyre::{Result, bail};
use parking_lot::RwLockReadGuard;
use tracing::error;

use crate::FnvIndexMap;
use crate::pickle::value::rw_arc::RwArc;

use super::Value;

#[derive(Clone)]
pub struct Dict(RwArc<FnvIndexMap<Element, Element>>);

impl Dict {
    pub fn new() -> Self {
        Self(RwArc::new(FnvIndexMap::default()))
    }

    pub fn insert(&self, key: impl Into<Value>, value: impl Into<Value>) -> Result<()> {
        let key = key.into();
        let value = value.into();

        if !key.is_hashable() {
            bail!("key is not hashable: {key:#?}");
        }

        self.0.write().insert(Element(key), Element(value));

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn get(&self, key: impl Into<Value>) -> Option<Value> {
        self.0
            .read()
            .get(&Element(key.into()))
            .map(|Element(value)| value)
            .cloned()
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }

    pub fn values(&self) -> Values {
        Values::new(self.clone())
    }

    pub fn read(&self) -> ReadDictGuard {
        ReadDictGuard::new(self)
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

impl IntoIterator for Dict {
    type Item = (Value, Value);
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl IntoIterator for &Dict {
    type Item = (Value, Value);
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
    }
}

pub struct Iter {
    dict: Dict,
    index: usize,
    max_len: usize,
}

impl Iter {
    fn new(dict: Dict) -> Self {
        Iter {
            max_len: dict.len(),
            dict,
            index: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = (Value, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let map = self.dict.0.read();

        // This prevents appending a dict to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let (Element(key), Element(value)) = map.get_index(self.index)?;
        let value = (key.clone(), value.clone());

        self.index += 1;

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max_len - self.index.min(self.max_len)))
    }
}

pub struct ReadDictGuard<'a> {
    dict: RwLockReadGuard<'a, FnvIndexMap<Element, Element>>,
}

impl<'a> ReadDictGuard<'a> {
    fn new(dict: &'a Dict) -> Self {
        let dict = dict.0.read();

        Self { dict }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.dict.iter().map(|(k, v)| (&k.0, &v.0))
    }
}

pub struct Values {
    dict: Dict,
    index: usize,
    max_len: usize,
}

impl Values {
    fn new(dict: Dict) -> Self {
        Self {
            max_len: dict.len(),
            dict,
            index: 0,
        }
    }
}

impl Iterator for Values {
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
            .unwrap_or_else(|err| error!("hash of unhashable value: {err}"));
    }
}

impl cmp::PartialEq for Element {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// This is a lie. All bets are off.
impl cmp::Eq for Element {}
