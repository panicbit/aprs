use std::hash::{Hash, Hasher};
use std::{fmt, ops};

#[derive(Clone, PartialEq, Eq)]
pub struct Bool(bool);

impl Bool {
    pub const fn r#false() -> Self {
        Bool(false)
    }

    pub const fn r#true() -> Self {
        Bool(true)
    }

    #[expect(non_snake_case)]
    pub const fn False() -> Self {
        Self::r#false()
    }

    #[expect(non_snake_case)]
    pub const fn True() -> Self {
        Self::r#true()
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
        f.debug_tuple("Bool").field(&self.0).finish()
    }
}

impl From<bool> for Bool {
    fn from(value: bool) -> Self {
        Bool(value)
    }
}
