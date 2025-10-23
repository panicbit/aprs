use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{f64, fmt};

use eyre::{Error, Result, bail};
use num::{BigInt, FromPrimitive, ToPrimitive, Zero};

use crate::pickle::value::Id;

use super::Value;

// TODO: ensure that all int types properly get represented as the smallest possible N type

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct Number(N, u64);

impl Number {
    // not public because construction of N needs to uphold its laws
    fn new(n: N) -> Self {
        Self(n, ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }

    pub fn id(&self) -> Id {
        Id::new_number(self.1).into()
    }

    pub fn inner(&self) -> &N {
        &self.0
    }

    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        let n = BigInt::from_signed_bytes_le(bytes);

        Number::from(n)
    }

    pub fn is_i64(&self) -> bool {
        matches!(self.0, N::I64(_))
    }

    pub fn is_i128(&self) -> bool {
        matches!(self.0, N::I128(_))
    }

    pub fn is_big_int(&self) -> bool {
        matches!(self.0, N::BigInt(_))
    }

    pub fn is_f64(&self) -> bool {
        matches!(self.0, N::F64(_))
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self.0 {
            N::I64(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_i128(&self) -> Option<i128> {
        match self.0 {
            N::I128(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_big_int(&self) -> Option<&BigInt> {
        match &self.0 {
            N::BigInt(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self.0 {
            N::F64(n) => Some(n),
            _ => None,
        }
    }

    pub fn add(&self, rhs: &Number) -> Number {
        Pair::from(self, rhs).add()
    }

    pub fn sub(&self, rhs: &Number) -> Number {
        Pair::from(self, rhs).sub()
    }

    pub fn mul(&self, rhs: &Number) -> Number {
        Pair::from(self, rhs).mul()
    }

    // pub fn pow(&self, rhs: &Number) -> Number {
    //     Pair::from(self, rhs).pow()
    // }

    pub fn or(&self, rhs: &Number) -> Result<Number> {
        Pair::from(self, rhs).or()
    }

    pub fn and(&self, rhs: &Number) -> Result<Number> {
        Pair::from(self, rhs).and()
    }

    pub fn floor(&self) -> Number {
        match self.inner() {
            N::I64(_) | N::I128(_) | N::BigInt(_) => self.clone(),
            N::F64(n) => Self::from(n.floor()),
        }
    }

    pub fn ceil(&self) -> Number {
        match self.inner() {
            N::I64(_) | N::I128(_) | N::BigInt(_) => self.clone(),
            N::F64(n) => Self::from(n.ceil()),
        }
    }

    pub fn to_usize(&self) -> Option<usize> {
        match *self.inner() {
            N::I64(n) => n.try_into().ok(),
            N::I128(n) => n.try_into().ok(),
            N::BigInt(ref n) => n.to_usize(),
            N::F64(_) => None,
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

impl From<i8> for Number {
    fn from(n: i8) -> Self {
        Self::new(N::from(i64::from(n)))
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

impl From<u64> for Number {
    fn from(n: u64) -> Self {
        Self::new(N::from(n))
    }
}

impl From<i128> for Number {
    fn from(n: i128) -> Self {
        Self::new(N::from(n))
    }
}

impl From<u128> for Number {
    fn from(n: u128) -> Self {
        Self::new(N::from(n))
    }
}

impl From<usize> for Number {
    fn from(n: usize) -> Self {
        Self::new(N::from(n))
    }
}

impl From<BigInt> for Number {
    fn from(n: BigInt) -> Self {
        Self::new(N::from(n))
    }
}

impl From<f32> for Number {
    fn from(n: f32) -> Self {
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

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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

impl From<i64> for N {
    fn from(n: i64) -> Self {
        N::I64(n)
    }
}

impl From<u64> for N {
    fn from(n: u64) -> Self {
        if let Ok(n) = i64::try_from(n) {
            return N::from(n);
        }

        N::I128(n.into())
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

impl From<u128> for N {
    fn from(n: u128) -> Self {
        if let Ok(n) = i128::try_from(n) {
            return N::from(n);
        }

        N::BigInt(n.into())
    }
}

impl From<usize> for N {
    fn from(n: usize) -> Self {
        if let Ok(n) = i128::try_from(n) {
            return N::from(n);
        }

        N::from(BigInt::from(n))
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

impl From<f32> for N {
    fn from(n: f32) -> Self {
        if n.fract().is_zero() {
            if let Some(n) = BigInt::from_f32(n) {
                return N::from(n);
            }
        }

        N::F64(n.into())
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
        match self.inner() {
            N::I64(n) => n.fmt(f),
            N::I128(n) => n.fmt(f),
            N::BigInt(n) => n.fmt(f),
            N::F64(n) => n.fmt(f),
        }
    }
}

enum Pair {
    I64(i64, i64),
    I128(i128, i128),
    BigInt(BigInt, BigInt),
    F64(f64, f64),
}

impl Pair {
    fn from(a: &Number, b: &Number) -> Self {
        match (a.inner().clone(), b.inner().clone()) {
            (N::I64(a), N::I64(b)) => Self::I64(a, b),
            (N::I64(a), N::I128(b)) => Self::I128(a as i128, b),
            (N::I64(a), N::BigInt(b)) => Self::BigInt(a.into(), b),
            (N::I64(a), N::F64(b)) => Self::F64(a as f64, b),
            (N::I128(a), N::I64(b)) => Self::I128(a, b.into()),
            (N::I128(a), N::I128(b)) => Self::I128(a, b),
            (N::I128(a), N::BigInt(b)) => Self::BigInt(a.into(), b),
            (N::I128(a), N::F64(b)) => Self::F64(a as f64, b),
            (N::BigInt(a), N::I64(b)) => Self::BigInt(a, b.into()),
            (N::BigInt(a), N::I128(b)) => Self::BigInt(a, b.into()),
            (N::BigInt(a), N::BigInt(b)) => Self::BigInt(a, b),
            (N::BigInt(a), N::F64(b)) => Self::F64(a.to_f64().unwrap_or(f64::NAN), b),
            (N::F64(a), N::I64(b)) => Self::F64(a, b as f64),
            (N::F64(a), N::I128(b)) => Self::F64(a, b as f64),
            (N::F64(a), N::BigInt(b)) => Self::F64(a, b.to_f64().unwrap_or(f64::NAN)),
            (N::F64(a), N::F64(b)) => Self::F64(a, b),
        }
    }

    fn add(self) -> Number {
        match self {
            Self::I64(a, b) => a
                .checked_add(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::I128(a.into(), b.into()).add()),
            Self::I128(a, b) => a
                .checked_add(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::BigInt(a.into(), b.into()).add()),
            Self::BigInt(a, b) => Number::from(a + b),
            Self::F64(a, b) => Number::from(a + b),
        }
    }

    fn sub(self) -> Number {
        match self {
            Self::I64(a, b) => a
                .checked_sub(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::I128(a as i128, b as i128).sub()),
            Self::I128(a, b) => a
                .checked_sub(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::BigInt(BigInt::from(a), BigInt::from(b)).sub()),
            Self::BigInt(a, b) => Number::from(a - b),
            Self::F64(a, b) => Number::from(a - b),
        }
    }

    fn mul(self) -> Number {
        match self {
            Self::I64(a, b) => a
                .checked_mul(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::I128(a as i128, b as i128).mul()),
            Self::I128(a, b) => a
                .checked_mul(b)
                .map(Number::from)
                .unwrap_or_else(|| Self::BigInt(BigInt::from(a), BigInt::from(b)).mul()),
            Self::BigInt(a, b) => Number::from(a * b),
            Self::F64(a, b) => Number::from(a * b),
        }
    }

    // fn pow(self) -> Number {
    //     match self {
    //         Self::I64(a, b) => a
    //             .checked_pow(b)
    //             .map(Number::from)
    //             .unwrap_or_else(|| Self::I128(a as i128, b as i128).pow()),
    //         Self::I128(a, b) => a
    //             .checked_pow(b)
    //             .map(Number::from)
    //             .unwrap_or_else(|| Self::BigInt(BigInt::from(a), BigInt::from(b)).pow()),
    //         Self::BigInt(a, b) => Number::from(a.pow(b)),
    //         Self::F64(a, b) => Number::from(a.powf(b)),
    //     }
    // }

    fn or(self) -> Result<Number> {
        Ok(match self {
            Pair::I64(a, b) => (a | b).into(),
            Pair::I128(a, b) => (a | b).into(),
            Pair::BigInt(a, b) => (a | b).into(),
            Pair::F64(_, _) => bail!("can't OR floats"),
        })
    }

    fn and(self) -> Result<Number> {
        Ok(match self {
            Pair::I64(a, b) => (a & b).into(),
            Pair::I128(a, b) => (a & b).into(),
            Pair::BigInt(a, b) => (a & b).into(),
            Pair::F64(_, _) => bail!("can't AND floats"),
        })
    }
}
