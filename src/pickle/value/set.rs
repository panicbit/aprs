use std::hash::{Hash, Hasher};
use std::{cmp, fmt};

use dumpster::Trace;
use eyre::{Result, bail};
use tracing::error;

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
            bail!("key is not hashable: {key:?}");
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

    pub fn iter(&self) -> Iter {
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

impl IntoIterator for Set {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

impl IntoIterator for &Set {
    type Item = Value;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self.clone())
    }
}

pub struct Iter {
    set: Set,
    index: usize,
    max_len: usize,
}

impl Iter {
    fn new(set: Set) -> Self {
        Self {
            max_len: set.len(),
            set,
            index: 0,
        }
    }
}

impl Iterator for Iter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let vec = self.set.0.read();

        // This prevents appending a set to itself from ending up in an endless loop
        if self.index >= self.max_len {
            return None;
        }

        let Element(value) = vec.get_index(self.index)?;
        let value = value.clone();

        self.index += 1;

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.max_len - self.index.min(self.max_len)))
    }
}

#[derive(Trace)]
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
