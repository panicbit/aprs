use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Clone)]
pub struct None(());

impl None {
    pub const fn new() -> Self {
        None(())
    }
}

impl Default for None {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for None {
    fn hash<H: Hasher>(&self, state: &mut H) {
        #[allow(clippy::unit_hash)]
        ().hash(state);
    }
}

impl fmt::Debug for None {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "None")
    }
}
