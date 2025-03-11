use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

use anyhow::{Result, bail};
use dumpster::Trace;

use crate::FnvIndexSet;
use crate::pickle::value::Id;
use crate::pickle::value::rw_gc::RwGc;
use crate::pickle::value::traced::Traced;

use super::Value;

#[derive(Trace, Clone)]
pub struct Set(RwGc<Traced<FnvIndexSet<Element>>>);

impl Set {
    pub fn new() -> Self {
        Self(RwGc::new(Traced(FnvIndexSet::default())))
    }

    pub fn id(&self) -> Id {
        self.0.id()
    }

    pub fn insert(&self, key: impl Into<Value>) -> Result<()> {
        let key = key.into();

        if !key.is_hashable() {
            bail!("key is not hashable");
        }

        self.0.write().insert(Element(key));

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

    pub fn contains(&self, key: Value) -> bool {
        self.0.read().contains(&Element(key))
    }

    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
    }
}

impl Default for Set {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Set {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for key in self {
            if !other.contains(key) {
                return false;
            }
        }

        for key in other {
            if !self.contains(key) {
                return false;
            }
        }

        true
    }
}

impl fmt::Debug for Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'a> IntoIterator for &'a Set {
    type Item = Value;
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
    dict: &'a Set,
    index: usize,
    max_len: usize,
}

impl Iterator for Iter<'_> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.dict.0.read();

        // This prevents appending a set to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let Element(value) = vec.get_index(self.index)?;
        let value = value.clone();

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
