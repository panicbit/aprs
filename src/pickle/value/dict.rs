use std::fmt;

use eyre::{Result, bail};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::{FnvIndexMap, pickle::value::rw_arc::RwArc};

use super::Value;

type Map = FnvIndexMap<Value, Value>;

#[derive(Clone)]
pub struct Dict(RwArc<Map>);

impl Dict {
    pub fn new() -> Self {
        Self(RwArc::new(Map::default()))
    }

    pub fn insert(&self, key: impl Into<Value>, value: impl Into<Value>) -> Result<()> {
        let key = key.into();
        let value = value.into();

        if !key.is_hashable() {
            bail!("key is not hashable: {key:#?}");
        }

        self.0.write().insert(key, value);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.0.read().len()
    }

    pub fn get(&self, key: &Value) -> Option<Value> {
        self.0.read().get(key).cloned()
    }

    pub fn read(&self) -> ReadDictGuard<'_> {
        ReadDictGuard::new(self)
    }

    pub fn write(&self) -> WriteDictGuard<'_> {
        WriteDictGuard::new(self)
    }

    pub fn update(&self, other: &Dict) -> Result<()> {
        if self.0.same_as(&other.0) {
            return Ok(());
        }

        let mut this = self.write();

        for (key, value) in other.read().iter() {
            let key = key.clone();
            let value = value.clone();

            this.insert(key, value)?;
        }

        Ok(())
    }
}

impl PartialEq for Dict {
    fn eq(&self, other: &Self) -> bool {
        let this = self.read();
        let other = other.read();

        if this.len() != other.len() {
            return false;
        }

        for (key, value) in this.iter() {
            if other.get(key) != Some(value) {
                return false;
            }
        }

        for (key, value) in other.iter() {
            if this.get(key) != Some(value) {
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
        f.debug_map().entries(self.read().iter()).finish()
    }
}

pub struct ReadDictGuard<'a> {
    dict: RwLockReadGuard<'a, Map>,
}

impl<'a> ReadDictGuard<'a> {
    fn new(dict: &'a Dict) -> Self {
        let dict = dict.0.read();

        Self { dict }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.dict.iter()
    }

    pub fn len(&self) -> usize {
        self.dict.len()
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        self.dict.get(key)
    }
}

pub struct WriteDictGuard<'a> {
    dict: RwLockWriteGuard<'a, Map>,
}

impl<'a> WriteDictGuard<'a> {
    fn new(dict: &'a Dict) -> Self {
        let dict = dict.0.write();

        Self { dict }
    }

    pub fn insert(&mut self, key: Value, value: Value) -> Result<()> {
        if !key.is_hashable() {
            bail!("key is not hashable: {key:#?}");
        }

        self.dict.insert(key, value);

        Ok(())
    }
}
