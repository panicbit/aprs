use std::ops::Deref;

use dumpster::Trace;

#[derive(Debug)]
pub struct BigInt(num::BigInt);

impl Deref for BigInt {
    type Target = num::BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Trace for BigInt {
    fn accept<V: dumpster::Visitor>(&self, _visitor: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

impl From<u8> for BigInt {
    fn from(value: u8) -> Self {
        Self(num::BigInt::from(value))
    }
}

impl From<&'_ u8> for BigInt {
    fn from(value: &u8) -> Self {
        Self(num::BigInt::from(*value))
    }
}

impl From<i32> for BigInt {
    fn from(value: i32) -> Self {
        Self(num::BigInt::from(value))
    }
}

impl From<&'_ i32> for BigInt {
    fn from(value: &i32) -> Self {
        Self(num::BigInt::from(*value))
    }
}

impl From<usize> for BigInt {
    fn from(value: usize) -> Self {
        Self(num::BigInt::from(value))
    }
}

impl From<&'_ usize> for BigInt {
    fn from(value: &usize) -> Self {
        Self(num::BigInt::from(*value))
    }
}

impl PartialEq for BigInt {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
