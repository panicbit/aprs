use dumpster::sync::Gc;
use fnv::FnvHashMap;

use crate::pickle::value::BigInt;

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
            .or_insert_with(|| Value::Byte(Gc::new(value)))
            .clone()
    }

    pub fn get_i16(&mut self, value: i16) -> Value {
        self.values
            .entry(value)
            .or_insert_with(|| Value::Int(Gc::new(value.into())))
            .clone()
    }

    pub fn get_i32(&mut self, value: i32) -> Value {
        if let Ok(value) = u8::try_from(value) {
            return self.get_u8(value);
        }

        if let Ok(value) = i16::try_from(value) {
            return self.get_i16(value);
        }

        Value::Int(Gc::new(value))
    }

    pub fn get_usize(&mut self, value: usize) -> Value {
        if let Ok(value) = i32::try_from(value) {
            return self.get_i32(value);
        }

        // TODO: consider using a u64
        let big_int = BigInt::from(value);

        Value::BigInt(Gc::new(big_int))
    }
}
