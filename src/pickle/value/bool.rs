use std::hash::{Hash, Hasher};
use std::sync::{Arc, LazyLock};
use std::{fmt, ops};

use crate::pickle::value::Id;

static FALSE: LazyLock<Bool> = LazyLock::new(|| Bool(Arc::new(false)));
static TRUE: LazyLock<Bool> = LazyLock::new(|| Bool(Arc::new(true)));

#[derive(Clone, PartialEq, Eq)]
pub struct Bool(Arc<bool>);

impl Bool {
    pub fn r#false() -> Self {
        FALSE.clone()
    }

    pub fn r#true() -> Self {
        TRUE.clone()
    }

    #[expect(non_snake_case)]
    pub fn False() -> Self {
        FALSE.clone()
    }

    #[expect(non_snake_case)]
    pub fn True() -> Self {
        TRUE.clone()
    }

    pub fn id(&self) -> Id {
        Arc::as_ptr(&self.0).into()
    }
}

impl ops::Deref for Bool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for Bool {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Debug for Bool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Bool").field(self.0.as_ref()).finish()
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        match value {
            true => Bool::r#true(),
            false => Bool::r#false(),
        }
    }
}
