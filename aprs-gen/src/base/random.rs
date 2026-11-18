use rand::rngs::{ChaCha12Rng, Xoshiro128PlusPlus};
use rand::{SeedableRng, TryRng};

use crate::base::Seed;

pub struct Random(Inner);

impl Random {
    pub fn seeded(seed: Seed) -> Self {
        let rng = Xoshiro128PlusPlus::from_seed(seed.into());

        Self(Inner::Seeded(rng))
    }

    pub fn secure() -> Self {
        let rng = rand::make_rng::<ChaCha12Rng>();

        Self(Inner::Secure(rng))
    }

    pub fn is_seeded(&self) -> bool {
        matches!(self.0, Inner::Seeded(_))
    }

    pub fn is_secure(&self) -> bool {
        matches!(self.0, Inner::Secure(_))
    }

    pub fn fork(&mut self) -> Random {
        match &mut self.0 {
            Inner::Seeded(rng) => Self(Inner::Seeded(rng.fork())),
            Inner::Secure(rng) => Self(Inner::Secure(rng.fork())),
        }
    }
}

impl TryRng for Random {
    type Error = !;

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        match &mut self.0 {
            Inner::Seeded(rng) => rng.try_next_u32(),
            Inner::Secure(rng) => rng.try_next_u32(),
        }
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        match &mut self.0 {
            Inner::Seeded(rng) => rng.try_next_u64(),
            Inner::Secure(rng) => rng.try_next_u64(),
        }
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Self::Error> {
        match &mut self.0 {
            Inner::Seeded(rng) => rng.try_fill_bytes(dst),
            Inner::Secure(rng) => rng.try_fill_bytes(dst),
        }
    }
}

impl From<Seed> for Random {
    fn from(seed: Seed) -> Self {
        Self::seeded(seed)
    }
}

pub enum Inner {
    Seeded(Xoshiro128PlusPlus),
    Secure(ChaCha12Rng),
}
