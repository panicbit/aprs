use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, Mul, Sub};

use eyre::{ContextCompat, Result, bail};
use num::{BigInt, FromPrimitive, One, Zero};

use crate::{Bool, Float, Storage, Value};

#[derive(Clone, Debug)]
pub enum Int {
    I64(i64),
    I128(i128),
    BigInt(BigInt),
}

impl Int {
    pub fn is_i64(&self) -> bool {
        matches!(self, Self::I64(_))
    }

    pub fn is_i128(&self) -> bool {
        matches!(self, Self::I128(_))
    }

    pub fn is_big_int(&self) -> bool {
        matches!(self, Self::BigInt(_))
    }

    fn is_zero(&self) -> bool {
        match self {
            Int::I64(n) => *n == 0,
            Int::I128(n) => *n == 0,
            Int::BigInt(n) => n.is_zero(),
        }
    }

    fn is_one(&self) -> bool {
        match self {
            Int::I64(n) => *n == 1,
            Int::I128(n) => *n == 1,
            Int::BigInt(n) => n.is_one(),
        }
    }

    pub fn from_signed_bytes_le(bytes: &[u8]) -> Self {
        match bytes.len() {
            1 => Self::I64(i8::from_le_bytes(bytes.try_into().unwrap()) as i64),
            2 => Self::I64(i16::from_le_bytes(bytes.try_into().unwrap()) as i64),
            4 => Self::I64(i32::from_le_bytes(bytes.try_into().unwrap()) as i64),
            8 => Self::I64(i64::from_le_bytes(bytes.try_into().unwrap())),
            16 => Self::I128(i128::from_le_bytes(bytes.try_into().unwrap())),
            _ => Self::BigInt(BigInt::from_signed_bytes_le(bytes)),
        }
    }

    pub fn to_usize(&self) -> Option<usize> {
        match *self {
            Int::I64(n) => n.try_into().ok(),
            Int::I128(n) => n.try_into().ok(),
            Int::BigInt(ref n) => n.try_into().ok(),
        }
    }
}

impl From<u8> for Int {
    fn from(value: u8) -> Self {
        Self::I64(value as u64 as i64)
    }
}

impl From<i8> for Int {
    fn from(value: i8) -> Self {
        Self::I64(value as i64)
    }
}

impl From<u16> for Int {
    fn from(value: u16) -> Self {
        Self::I64(value.into())
    }
}

impl From<i16> for Int {
    fn from(value: i16) -> Self {
        Self::I64(value.into())
    }
}

impl From<u32> for Int {
    fn from(value: u32) -> Self {
        Self::I64(value.into())
    }
}

impl From<i32> for Int {
    fn from(value: i32) -> Self {
        Self::I64(value.into())
    }
}

impl From<u64> for Int {
    fn from(value: u64) -> Self {
        (i64::try_from(value).map(Self::I64)).unwrap_or_else(|_| Self::I128(value.into()))
    }
}

impl From<i64> for Int {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u128> for Int {
    fn from(value: u128) -> Self {
        (i128::try_from(value).map(Self::I128))
            .unwrap_or_else(|_| Self::BigInt(BigInt::from(value)))
    }
}

impl From<i128> for Int {
    fn from(value: i128) -> Self {
        Self::I128(value)
    }
}

impl TryFrom<f64> for Int {
    type Error = eyre::Report;

    fn try_from(value: f64) -> Result<Self> {
        if i64::MIN as f64 <= value && value <= i64::MAX as f64 {
            debug_assert!(value.is_finite());
            return Ok(Self::I64(value as i64));
        }

        if i128::MIN as f64 <= value && value <= i128::MAX as f64 {
            debug_assert!(value.is_finite());
            return Ok(Self::I128(value as i128));
        }

        let value = BigInt::from_f64(value)
            .with_context(|| format!("cannot convert float {} to integer", value))?;

        Ok(Self::BigInt(value))
    }
}

impl From<usize> for Int {
    fn from(value: usize) -> Self {
        (i64::try_from(value).map(Self::I64))
            .or_else(|_| i128::try_from(value).map(Self::I128))
            .unwrap_or_else(|_| Self::BigInt(BigInt::from(value)))
    }
}

impl TryFrom<Float> for Int {
    type Error = eyre::Report;

    fn try_from(value: Float) -> Result<Self> {
        (*value).try_into()
    }
}

impl<S: Storage> TryFrom<Value<S>> for Int {
    type Error = eyre::Report;

    fn try_from(value: Value<S>) -> Result<Self, Self::Error> {
        match value {
            Value::Int(n) => Ok(n),
            Value::Float(n) => Ok(n.try_into()?),
            Value::Dict(_)
            | Value::List(_)
            | Value::Str(_)
            | Value::Bool(_)
            | Value::Tuple(_)
            | Value::Callable(_)
            | Value::None(_)
            | Value::Set(_) => bail!("can't convert {} to Int", value.type_name()),
        }
    }
}

impl Add<Int> for Int {
    type Output = Int;

    fn add(self, rhs: Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => add_i64(a, b)
                .or_else(|| add_i128(a, b))
                .unwrap_or_else(|| add_big_int(a, b)),
            (Int::I64(a), Int::I128(b)) => add_i128(a, b).unwrap_or_else(|| add_big_int(a, b)),
            (Int::I64(a), Int::BigInt(b)) => add_big_int(a, b),
            (Int::I128(a), Int::I64(b)) => add_i128(a, b).unwrap_or_else(|| add_big_int(a, b)),
            (Int::I128(a), Int::I128(b)) => add_i128(a, b).unwrap_or_else(|| add_big_int(a, b)),
            (Int::I128(a), Int::BigInt(b)) => add_big_int(a, b),
            (Int::BigInt(a), Int::I64(b)) => add_big_int(a, b),
            (Int::BigInt(a), Int::I128(b)) => add_big_int(a, b),
            (Int::BigInt(a), Int::BigInt(b)) => add_big_int(a, b),
        }
    }
}

// TODO: optimize / prevent unnecessary clones
impl Add<&Int> for &Int {
    type Output = Int;

    fn add(self, rhs: &Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => add_i64(*a, *b)
                .or_else(|| add_i128(*a, *b))
                .unwrap_or_else(|| add_big_int(*a, *b)),
            (Int::I64(a), Int::I128(b)) => add_i128(*a, *b).unwrap_or_else(|| add_big_int(*a, *b)),
            (Int::I64(a), Int::BigInt(b)) => add_big_int(*a, b.clone()),
            (Int::I128(a), Int::I64(b)) => add_i128(*a, *b).unwrap_or_else(|| add_big_int(*a, *b)),
            (Int::I128(a), Int::I128(b)) => add_i128(*a, *b).unwrap_or_else(|| add_big_int(*a, *b)),
            (Int::I128(a), Int::BigInt(b)) => add_big_int(*a, b.clone()),
            (Int::BigInt(a), Int::I64(b)) => add_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::I128(b)) => add_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::BigInt(b)) => add_big_int(a.clone(), b.clone()),
        }
    }
}

fn add_i64(a: impl Into<i64>, b: impl Into<i64>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_add(b).map(Int::I64)
}

fn add_i128(a: impl Into<i128>, b: impl Into<i128>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_add(b).map(Int::I128)
}

// TODO: optimization: only one operand needs to be converted BigInt
fn add_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::BigInt(a.add(b))
}

impl Sub<Int> for Int {
    type Output = Int;

    fn sub(self, rhs: Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => sub_i64(a, b)
                .or_else(|| sub_i128(a, b))
                .unwrap_or_else(|| sub_big_int(a, b)),
            (Int::I64(a), Int::I128(b)) => sub_i128(a, b).unwrap_or_else(|| sub_big_int(a, b)),
            (Int::I64(a), Int::BigInt(b)) => sub_big_int(a, b),
            (Int::I128(a), Int::I64(b)) => sub_i128(a, b).unwrap_or_else(|| sub_big_int(a, b)),
            (Int::I128(a), Int::I128(b)) => sub_i128(a, b).unwrap_or_else(|| sub_big_int(a, b)),
            (Int::I128(a), Int::BigInt(b)) => sub_big_int(a, b),
            (Int::BigInt(a), Int::I64(b)) => sub_big_int(a, b),
            (Int::BigInt(a), Int::I128(b)) => sub_big_int(a, b),
            (Int::BigInt(a), Int::BigInt(b)) => sub_big_int(a, b),
        }
    }
}

// TODO: optimize / prevent unnecessary clones
impl Sub<&Int> for &Int {
    type Output = Int;

    fn sub(self, rhs: &Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => add_i64(*a, *b)
                .or_else(|| sub_i128(*a, *b))
                .unwrap_or_else(|| sub_big_int(*a, *b)),
            (Int::I64(a), Int::I128(b)) => sub_i128(*a, *b).unwrap_or_else(|| sub_big_int(*a, *b)),
            (Int::I64(a), Int::BigInt(b)) => sub_big_int(*a, b.clone()),
            (Int::I128(a), Int::I64(b)) => sub_i128(*a, *b).unwrap_or_else(|| sub_big_int(*a, *b)),
            (Int::I128(a), Int::I128(b)) => sub_i128(*a, *b).unwrap_or_else(|| sub_big_int(*a, *b)),
            (Int::I128(a), Int::BigInt(b)) => sub_big_int(*a, b.clone()),
            (Int::BigInt(a), Int::I64(b)) => sub_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::I128(b)) => sub_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::BigInt(b)) => sub_big_int(a.clone(), b.clone()),
        }
    }
}

fn sub_i64(a: impl Into<i64>, b: impl Into<i64>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_sub(b).map(Int::I64)
}

fn sub_i128(a: impl Into<i128>, b: impl Into<i128>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_sub(b).map(Int::I128)
}

// TODO: optimization: only one operand needs to be converted BigInt
fn sub_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::BigInt(a.sub(b))
}

impl Mul<Int> for Int {
    type Output = Int;

    fn mul(self, rhs: Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => mul_i64(a, b)
                .or_else(|| mul_i128(a, b))
                .unwrap_or_else(|| mul_big_int(a, b)),
            (Int::I64(a), Int::I128(b)) => mul_i128(a, b).unwrap_or_else(|| mul_big_int(a, b)),
            (Int::I64(a), Int::BigInt(b)) => mul_big_int(a, b),
            (Int::I128(a), Int::I64(b)) => mul_i128(a, b).unwrap_or_else(|| mul_big_int(a, b)),
            (Int::I128(a), Int::I128(b)) => mul_i128(a, b).unwrap_or_else(|| mul_big_int(a, b)),
            (Int::I128(a), Int::BigInt(b)) => mul_big_int(a, b),
            (Int::BigInt(a), Int::I64(b)) => mul_big_int(a, b),
            (Int::BigInt(a), Int::I128(b)) => mul_big_int(a, b),
            (Int::BigInt(a), Int::BigInt(b)) => mul_big_int(a, b),
        }
    }
}

// TODO: optimize / prevent unnecessary clones
impl Mul<&Int> for &Int {
    type Output = Int;

    fn mul(self, rhs: &Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => mul_i64(*a, *b)
                .or_else(|| mul_i128(*a, *b))
                .unwrap_or_else(|| mul_big_int(*a, *b)),
            (Int::I64(a), Int::I128(b)) => mul_i128(*a, *b).unwrap_or_else(|| mul_big_int(*a, *b)),
            (Int::I64(a), Int::BigInt(b)) => mul_big_int(*a, b.clone()),
            (Int::I128(a), Int::I64(b)) => mul_i128(*a, *b).unwrap_or_else(|| mul_big_int(*a, *b)),
            (Int::I128(a), Int::I128(b)) => mul_i128(*a, *b).unwrap_or_else(|| mul_big_int(*a, *b)),
            (Int::I128(a), Int::BigInt(b)) => mul_big_int(*a, b.clone()),
            (Int::BigInt(a), Int::I64(b)) => mul_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::I128(b)) => mul_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::BigInt(b)) => mul_big_int(a.clone(), b.clone()),
        }
    }
}

fn mul_i64(a: impl Into<i64>, b: impl Into<i64>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_mul(b).map(Int::I64)
}

fn mul_i128(a: impl Into<i128>, b: impl Into<i128>) -> Option<Int> {
    let a = a.into();
    let b = b.into();

    a.checked_mul(b).map(Int::I128)
}

// TODO: optimization: only one operand needs to be converted BigInt
fn mul_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::BigInt(a.mul(b))
}

impl BitAnd<Int> for Int {
    type Output = Int;

    fn bitand(self, rhs: Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => and_i64(a, b),
            (Int::I64(a), Int::I128(b)) => and_i128(a, b),
            (Int::I64(a), Int::BigInt(b)) => and_big_int(a, b),
            (Int::I128(a), Int::I64(b)) => and_i128(a, b),
            (Int::I128(a), Int::I128(b)) => and_i128(a, b),
            (Int::I128(a), Int::BigInt(b)) => and_big_int(a, b),
            (Int::BigInt(a), Int::I64(b)) => and_big_int(a, b),
            (Int::BigInt(a), Int::I128(b)) => and_big_int(a, b),
            (Int::BigInt(a), Int::BigInt(b)) => and_big_int(a, b),
        }
    }
}

// TODO: optimize / prevent unnecessary clones
impl BitAnd<&Int> for &Int {
    type Output = Int;

    fn bitand(self, rhs: &Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => and_i64(*a, *b),
            (Int::I64(a), Int::I128(b)) => and_i128(*a, *b),
            (Int::I64(a), Int::BigInt(b)) => and_big_int(*a, b.clone()),
            (Int::I128(a), Int::I64(b)) => and_i128(*a, *b),
            (Int::I128(a), Int::I128(b)) => and_i128(*a, *b),
            (Int::I128(a), Int::BigInt(b)) => and_big_int(*a, b.clone()),
            (Int::BigInt(a), Int::I64(b)) => and_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::I128(b)) => and_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::BigInt(b)) => and_big_int(a.clone(), b.clone()),
        }
    }
}

fn and_i64(a: impl Into<i64>, b: impl Into<i64>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::I64(a.bitand(b))
}

fn and_i128(a: impl Into<i128>, b: impl Into<i128>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::I128(a.bitand(b))
}

// TODO: optimization: only one operand needs to be converted BigInt
fn and_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::BigInt(a.bitand(b))
}

impl BitOr<Int> for Int {
    type Output = Int;

    fn bitor(self, rhs: Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => or_i64(a, b),
            (Int::I64(a), Int::I128(b)) => or_i128(a, b),
            (Int::I64(a), Int::BigInt(b)) => or_big_int(a, b),
            (Int::I128(a), Int::I64(b)) => or_i128(a, b),
            (Int::I128(a), Int::I128(b)) => or_i128(a, b),
            (Int::I128(a), Int::BigInt(b)) => or_big_int(a, b),
            (Int::BigInt(a), Int::I64(b)) => or_big_int(a, b),
            (Int::BigInt(a), Int::I128(b)) => or_big_int(a, b),
            (Int::BigInt(a), Int::BigInt(b)) => or_big_int(a, b),
        }
    }
}

// TODO: optimize / prevent unnecessary clones
impl BitOr<&Int> for &Int {
    type Output = Int;

    fn bitor(self, rhs: &Int) -> Self::Output {
        match (self, rhs) {
            (Int::I64(a), Int::I64(b)) => or_i64(*a, *b),
            (Int::I64(a), Int::I128(b)) => or_i128(*a, *b),
            (Int::I64(a), Int::BigInt(b)) => or_big_int(*a, b.clone()),
            (Int::I128(a), Int::I64(b)) => or_i128(*a, *b),
            (Int::I128(a), Int::I128(b)) => or_i128(*a, *b),
            (Int::I128(a), Int::BigInt(b)) => or_big_int(*a, b.clone()),
            (Int::BigInt(a), Int::I64(b)) => or_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::I128(b)) => or_big_int(a.clone(), *b),
            (Int::BigInt(a), Int::BigInt(b)) => or_big_int(a.clone(), b.clone()),
        }
    }
}

fn or_i64(a: impl Into<i64>, b: impl Into<i64>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::I64(a.bitor(b))
}

fn or_i128(a: impl Into<i128>, b: impl Into<i128>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::I128(a.bitor(b))
}

// TODO: optimization: only one operand needs to be converted BigInt
fn or_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> Int {
    let a = a.into();
    let b = b.into();

    Int::BigInt(a.bitor(b))
}

impl PartialEq for Int {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Int::I64(a), Int::I64(b)) => a == b,
            (Int::I64(a), Int::I128(b)) => *a as i128 == *b,
            (Int::I64(a), Int::BigInt(b)) => BigInt::from(*a) == *b,
            (Int::I128(a), Int::I64(b)) => *a == *b as i128,
            (Int::I128(a), Int::I128(b)) => a == b,
            (Int::I128(a), Int::BigInt(b)) => BigInt::from(*a) == *b,
            (Int::BigInt(a), Int::I64(b)) => *a == BigInt::from(*b),
            (Int::BigInt(a), Int::I128(b)) => *a == BigInt::from(*b),
            (Int::BigInt(a), Int::BigInt(b)) => a == b,
        }
    }
}

fn eq_i64(a: impl Into<i64>, b: impl Into<i64>) -> bool {
    let a = a.into();
    let b = b.into();

    a.eq(&b)
}

fn eq_i128(a: impl Into<i128>, b: impl Into<i128>) -> bool {
    let a = a.into();
    let b = b.into();

    a.eq(&b)
}

// TODO: optimization: only one operand needs to be converted BigInt
fn eq_big_int(a: impl Into<BigInt>, b: impl Into<BigInt>) -> bool {
    let a = a.into();
    let b = b.into();

    a.eq(&b)
}

impl PartialEq<Float> for Int {
    fn eq(&self, other: &Float) -> bool {
        other.eq(self)
    }
}

impl PartialEq<Bool> for Int {
    fn eq(&self, other: &Bool) -> bool {
        match **other {
            true => self.is_one(),
            false => self.is_zero(),
        }
    }
}

impl Hash for Int {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match *self {
            Int::I64(n) => hash_i64(n, state),
            Int::I128(n) => hash_i128(n, state),
            Int::BigInt(ref n) => hash_big_int(n, state),
        }
    }
}

fn hash_i64(n: i64, state: &mut impl Hasher) {
    n.hash(state);
}

fn hash_i128(n: i128, state: &mut impl Hasher) {
    let Ok(n) = i64::try_from(n) else {
        n.hash(state);
        return;
    };

    hash_i64(n, state);
}

fn hash_big_int(n: &BigInt, state: &mut impl Hasher) {
    let Ok(n) = i128::try_from(n) else {
        n.hash(state);
        return;
    };

    hash_i128(n, state);
}
