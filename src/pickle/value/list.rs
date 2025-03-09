use std::{fmt, ops};

use anyhow::{Result, bail};
use dumpster::Trace;
use dumpster::sync::Gc;
use parking_lot::RwLock;

use super::Value;

pub struct List(RwLock<Vec<Value>>);

impl List {
    pub fn new() -> Gc<Self> {
        Gc::new(Self(RwLock::new(Vec::new())))
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

    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.read().get(index).cloned()
    }

    pub fn append(&self, values: Value) -> Result<()> {
        match values {
            Value::List(values) => self.append_list(&values),
            Value::Dict(_) => bail!("can't append Dict to List"),
            Value::BinStr(_) => bail!("can't append BinStr to List"),
            Value::Byte(_) => bail!("can't append Byte to List"),
            Value::Int(_) => bail!("can't append Int to List"),
            Value::BigInt(_) => bail!("can't append BigInt to List"),
        }
    }

    fn append_list(&self, list: &List) -> Result<()> {
        for value in list {
            self.push(value);
        }

        Ok(())
    }
}

unsafe impl Trace for List {
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        for value in self {
            value.accept(visitor)?;
        }

        Ok(())
    }
}

impl<'a> IntoIterator for &'a List {
    type Item = Value;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            list: self,
            index: 0,
            max_len: self.len(),
        }
    }
}

pub struct Iter<'a> {
    list: &'a List,
    index: usize,
    max_len: usize,
}

impl Iterator for Iter<'_> {
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
