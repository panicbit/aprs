use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::LazyLock;

use dumpster::Trace;
use dumpster::sync::Gc;

use crate::pickle::value::Id;

static NONE: LazyLock<None> = LazyLock::new(|| None(Gc::new(())));

#[derive(Clone, Trace)]
pub struct None(Gc<()>);

impl None {
    pub fn new() -> Self {
        NONE.clone()
    }

    pub fn id(&self) -> Id {
        Gc::as_ptr(&self.0).into()
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
