use anyhow::{Error, Result};
use dumpster::Trace;
use dumpster::sync::Gc;
use num::{BigInt, FromPrimitive, Zero};

use super::Value;

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct Number(Inner);

impl Number {
    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        let n = BigInt::from_signed_bytes_le(bytes);

        Number::from(n)
    }

    pub fn is_i64(&self) -> bool {
        matches!(self.0, Inner::I64(_))
    }

    pub fn is_i128(&self) -> bool {
        matches!(self.0, Inner::I128(_))
    }

    pub fn is_big_int(&self) -> bool {
        matches!(self.0, Inner::BigInt(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self.0, Inner::F64(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.0 {
            Inner::I64(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match self.0 {
            Inner::I128(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_big_int(&self) -> Option<&BigInt> {
        match &self.0 {
            Inner::BigInt(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.0 {
            Inner::F64(n) => Some(n),
            _ => None,
        }
    }
}

impl From<bool> for Number {
    fn from(value: bool) -> Self {
        Number::from(value as u8)
    }
}

impl From<u8> for Number {
    fn from(n: u8) -> Self {
        Self(Inner::from(i64::from(n)))
    }
}

impl From<i16> for Number {
    fn from(n: i16) -> Self {
        Self(Inner::from(i64::from(n)))
    }
}

impl From<u16> for Number {
    fn from(n: u16) -> Self {
        Self(Inner::from(i64::from(n)))
    }
}

impl From<i32> for Number {
    fn from(n: i32) -> Self {
        Self(Inner::from(i64::from(n)))
    }
}

impl From<u32> for Number {
    fn from(n: u32) -> Self {
        Self(Inner::from(i64::from(n)))
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self(Inner::from(n))
    }
}

impl From<i128> for Number {
    fn from(n: i128) -> Self {
        Self(Inner::from(n))
    }
}

impl From<usize> for Number {
    fn from(n: usize) -> Self {
        if let Ok(n) = i128::try_from(n) {
            return Self(Inner::from(n));
        }

        Self(Inner::from(BigInt::from(n)))
    }
}

impl From<BigInt> for Number {
    fn from(n: BigInt) -> Self {
        Self(Inner::from(n))
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Self(Inner::from(n))
    }
}

impl TryFrom<Value> for Gc<Number> {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        value.as_number()
    }
}

unsafe impl Trace for Number {
    fn accept<V: dumpster::Visitor>(&self, _visitor: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

/// This type guarantees to be the smallest integer type possible.
/// Floats get converted to integers if possible.
#[derive(Debug, Clone)]
enum Inner {
    I64(i64),
    I128(i128),
    BigInt(BigInt),
    F64(f64),
}

unsafe impl Trace for Inner {
    fn accept<V: dumpster::Visitor>(&self, _visitor: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

impl From<i64> for Inner {
    fn from(n: i64) -> Self {
        Inner::I64(n)
    }
}

impl From<i128> for Inner {
    fn from(n: i128) -> Self {
        if let Ok(n) = i64::try_from(n) {
            return Inner::from(n);
        }

        Inner::I128(n)
    }
}

impl From<BigInt> for Inner {
    fn from(n: BigInt) -> Self {
        if let Ok(n) = i128::try_from(&n) {
            return Inner::from(n);
        }

        Inner::BigInt(n)
    }
}

impl From<f64> for Inner {
    fn from(n: f64) -> Self {
        if n.fract().is_zero() {
            if let Some(n) = BigInt::from_f64(n) {
                return Inner::from(n);
            }
        }

        Inner::F64(n)
    }
}

impl PartialEq for Inner {
    fn eq(&self, other: &Self) -> bool {
        // This only work because Number always uses the smallest integer type possible,
        // or in the case of f64, tries to be an integer.
        match (self, other) {
            (Inner::I64(a), Inner::I64(b)) => a == b,
            (Inner::I128(a), Inner::I128(b)) => a == b,
            (Inner::BigInt(a), Inner::BigInt(b)) => a == b,
            (Inner::F64(a), Inner::F64(b)) => a == b,
            _ => false,
        }
    }
}

impl std::hash::Hash for Inner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // See PartialEq for why this implementation is correct.
        match self {
            Inner::I64(n) => n.hash(state),
            Inner::I128(n) => n.hash(state),
            // This is dumb, but oh well.
            Inner::F64(n) => (*n as u64).hash(state),
            Inner::BigInt(n) => n.hash(state),
        }
    }
}
