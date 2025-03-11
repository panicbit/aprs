use std::fmt;
use std::hash::{Hash, Hasher};

use anyhow::{Error, Result};
use dumpster::Trace;
use dumpster::sync::Gc;
use num::{BigInt, FromPrimitive, Zero};

use crate::pickle::value::Id;

use super::Value;

#[derive(Clone, PartialEq)]
pub struct Number(Gc<N>);

impl Number {
    // not public because construction of N needs to uphold its laws
    fn new(n: N) -> Self {
        Self(Gc::new(n))
    }

    pub fn id(&self) -> Id {
        Gc::as_ptr(&self.0).into()
    }

    pub fn inner(&self) -> &N {
        &self.0
    }

    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        let n = BigInt::from_signed_bytes_le(bytes);

        Number::from(n)
    }

    pub fn is_i64(&self) -> bool {
        matches!(*self.0, N::I64(_))
    }

    pub fn is_i128(&self) -> bool {
        matches!(*self.0, N::I128(_))
    }

    pub fn is_big_int(&self) -> bool {
        matches!(*self.0, N::BigInt(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(*self.0, N::F64(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self.0 {
            N::I64(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match *self.0 {
            N::I128(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_big_int(&self) -> Option<&BigInt> {
        match &*self.0 {
            N::BigInt(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match *self.0 {
            N::F64(n) => Some(n),
            _ => None,
        }
    }
}

impl Hash for Number {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl From<bool> for Number {
    fn from(value: bool) -> Self {
        Number::from(value as u8)
    }
}

impl From<u8> for Number {
    fn from(n: u8) -> Self {
        Self::new(N::from(i64::from(n)))
    }
}

impl From<i16> for Number {
    fn from(n: i16) -> Self {
        Self::new(N::from(i64::from(n)))
    }
}

impl From<u16> for Number {
    fn from(n: u16) -> Self {
        Self::new(N::from(i64::from(n)))
    }
}

impl From<i32> for Number {
    fn from(n: i32) -> Self {
        Self::new(N::from(i64::from(n)))
    }
}

impl From<u32> for Number {
    fn from(n: u32) -> Self {
        Self::new(N::from(i64::from(n)))
    }
}

impl From<i64> for Number {
    fn from(n: i64) -> Self {
        Self::new(N::from(n))
    }
}

impl From<i128> for Number {
    fn from(n: i128) -> Self {
        Self::new(N::from(n))
    }
}

impl From<usize> for Number {
    fn from(n: usize) -> Self {
        if let Ok(n) = i128::try_from(n) {
            return Self::new(N::from(n));
        }

        Self::new(N::from(BigInt::from(n)))
    }
}

impl From<BigInt> for Number {
    fn from(n: BigInt) -> Self {
        Self::new(N::from(n))
    }
}

impl From<f64> for Number {
    fn from(n: f64) -> Self {
        Self::new(N::from(n))
    }
}

impl TryFrom<Value> for Number {
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
/// This enum is not to be constructed manually
#[derive(Debug, Clone)]
pub enum N {
    I64(i64),
    I128(i128),
    BigInt(BigInt),
    F64(f64),
}

unsafe impl Trace for N {
    fn accept<V: dumpster::Visitor>(&self, _visitor: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

impl From<i64> for N {
    fn from(n: i64) -> Self {
        N::I64(n)
    }
}

impl From<i128> for N {
    fn from(n: i128) -> Self {
        if let Ok(n) = i64::try_from(n) {
            return N::from(n);
        }

        N::I128(n)
    }
}

impl From<BigInt> for N {
    fn from(n: BigInt) -> Self {
        if let Ok(n) = i128::try_from(&n) {
            return N::from(n);
        }

        N::BigInt(n)
    }
}

impl From<f64> for N {
    fn from(n: f64) -> Self {
        if n.fract().is_zero() {
            if let Some(n) = BigInt::from_f64(n) {
                return N::from(n);
            }
        }

        N::F64(n)
    }
}

impl PartialEq for N {
    fn eq(&self, other: &Self) -> bool {
        // This only work because Number always uses the smallest integer type possible,
        // or in the case of f64, tries to be an integer.
        match (self, other) {
            (N::I64(a), N::I64(b)) => a == b,
            (N::I128(a), N::I128(b)) => a == b,
            (N::BigInt(a), N::BigInt(b)) => a == b,
            (N::F64(a), N::F64(b)) => a == b,
            _ => false,
        }
    }
}

impl std::hash::Hash for N {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // See PartialEq for why this implementation is correct.
        match self {
            N::I64(n) => n.hash(state),
            N::I128(n) => n.hash(state),
            // This is dumb, but oh well.
            N::F64(n) => (*n as u64).hash(state),
            N::BigInt(n) => n.hash(state),
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_ref().fmt(f)
    }
}
