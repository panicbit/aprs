use fnv::FnvHashMap;

use crate::pickle::value::{Number, Storage};

use super::Value;

pub struct NumberCache<S: Storage> {
    values: FnvHashMap<i16, Value<S>>,
}

impl<S: Storage> NumberCache<S> {
    pub fn new() -> Self {
        NumberCache {
            values: FnvHashMap::default(),
        }
    }

    pub fn get_u8(&mut self, value: u8) -> Value<S> {
        self.values
            .entry(value.into())
            .or_insert_with(|| Value::Number(value.into()))
            .clone()
    }

    pub fn get_i16(&mut self, value: i16) -> Value<S> {
        self.values
            .entry(value)
            .or_insert_with(|| Value::Number(value.into()))
            .clone()
    }

    pub fn get_u16(&mut self, value: u16) -> Value<S> {
        if let Ok(value) = i16::try_from(value) {
            return self.get_i16(value);
        }

        Value::Number(Number::from(value))
    }

    pub fn get_i32(&mut self, value: i32) -> Value<S> {
        if let Ok(value) = i16::try_from(value) {
            return self.get_i16(value);
        }

        Value::Number(Number::from(value))
    }

    pub fn get_u32(&mut self, value: u32) -> Value<S> {
        if let Ok(value) = u16::try_from(value) {
            return self.get_u16(value);
        }

        Value::Number(Number::from(value))
    }

    pub fn get_usize(&mut self, value: usize) -> Value<S> {
        if let Ok(value) = i32::try_from(value) {
            return self.get_i32(value);
        }

        Value::Number(Number::from(value))
    }
}
