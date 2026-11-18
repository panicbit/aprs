use std::fmt;

use rand::{RngExt, rng};

#[derive(Copy, Clone, Debug)]
pub struct Seed(u128);

impl Seed {
    pub fn new_random() -> Seed {
        Self(rng().random::<u128>())
    }

    pub fn to_u128(self) -> u128 {
        self.0
    }

    pub fn to_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl From<u128> for Seed {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl From<Seed> for [u8; 16] {
    fn from(value: Seed) -> Self {
        value.to_bytes()
    }
}

impl fmt::Display for Seed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}
