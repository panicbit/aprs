use std::ops::{Add, Deref, DerefMut, Mul, Sub};

use eyre::{Result, ensure};
use num::{One, ToPrimitive, Zero};

use crate::pickle::value::{Bool, Int};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Float(f64);

impl Float {
    pub fn floor(&self) -> Result<Int> {
        self.0.floor().try_into()
    }

    pub fn ceil(&self) -> Result<Int> {
        self.0.ceil().try_into()
    }
}

impl Deref for Float {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Float {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<f32> for Float {
    fn from(value: f32) -> Self {
        Self(value.into())
    }
}

impl From<f64> for Float {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl Add<Float> for Float {
    type Output = Float;

    fn add(self, rhs: Float) -> Self::Output {
        Self(self.0.add(rhs.0))
    }
}

impl Sub<Float> for Float {
    type Output = Float;

    fn sub(self, rhs: Float) -> Self::Output {
        Self(self.0.sub(rhs.0))
    }
}

impl Mul<Float> for Float {
    type Output = Float;

    fn mul(self, rhs: Float) -> Self::Output {
        Self(self.0.mul(rhs.0))
    }
}

impl From<&Float> for Float {
    fn from(n: &Float) -> Self {
        *n
    }
}

impl TryFrom<Int> for Float {
    type Error = eyre::Report;

    fn try_from(n: Int) -> Result<Self, Self::Error> {
        match n {
            Int::I64(n) => Ok(Self(n as f64)),
            Int::I128(n) => Ok(Self(n as f64)),
            Int::BigInt(n) => {
                let n = n.to_f64().unwrap_or(f64::INFINITY);

                ensure!(n.is_finite(), "Int too large to convert to Float");

                Ok(Self(n))
            }
        }
    }
}

impl TryFrom<&Int> for Float {
    type Error = eyre::Report;

    fn try_from(n: &Int) -> Result<Self, Self::Error> {
        match *n {
            Int::I64(n) => Ok(Self(n as f64)),
            Int::I128(n) => Ok(Self(n as f64)),
            Int::BigInt(ref n) => {
                let n = n.to_f64().unwrap_or(f64::INFINITY);

                ensure!(n.is_finite(), "Int too large to convert to Float");

                Ok(Self(n))
            }
        }
    }
}

impl PartialEq<Int> for Float {
    fn eq(&self, other: &Int) -> bool {
        let Ok(other) = Float::try_from(other) else {
            return false;
        };

        self.eq(&other)
    }
}

impl PartialEq<Bool> for Float {
    fn eq(&self, other: &Bool) -> bool {
        match **other {
            true => self.0.is_zero(),
            false => self.0.is_one(),
        }
    }
}
