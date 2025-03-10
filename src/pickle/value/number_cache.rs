use dumpster::sync::Gc;
use fnv::FnvHashMap;

use crate::pickle::value::Number;

use super::Value;

pub struct NumberCache {
    values: FnvHashMap<i16, Value>,
}

impl NumberCache {
    pub fn new() -> Self {
        NumberCache {
            values: FnvHashMap::default(),
        }
    }

    pub fn get_u8(&mut self, value: u8) -> Value {
        self.values
            .entry(value.into())
            .or_insert_with(|| Value::Number(Gc::new(value.into())))
            .clone()
    }

    pub fn get_i16(&mut self, value: i16) -> Value {
        self.values
            .entry(value)
            .or_insert_with(|| Value::Number(Gc::new(value.into())))
            .clone()
    }

    pub fn get_u16(&mut self, value: u16) -> Value {
        if let Ok(value) = i16::try_from(value) {
            return self.get_i16(value);
        }

        Value::Number(Gc::new(Number::from(value)))
    }

    pub fn get_i32(&mut self, value: i32) -> Value {
        if let Ok(value) = i16::try_from(value) {
            return self.get_i16(value);
        }

        Value::Number(Gc::new(Number::from(value)))
    }

    pub fn get_usize(&mut self, value: usize) -> Value {
        if let Ok(value) = i32::try_from(value) {
            return self.get_i32(value);
        }

        Value::Number(Gc::new(Number::from(value)))
    }
}
