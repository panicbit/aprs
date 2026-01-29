use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use eyre::{Context, Result};

use crate::Tuple;

use super::Value;

#[derive(Clone)]
#[expect(clippy::type_complexity)]
pub struct Callable(Arc<dyn Fn(&Tuple) -> Result<Value> + Send + Sync>);

impl Callable {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Tuple) -> Result<Value> + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }

    pub fn call(&self, args: Value) -> Result<Value> {
        let args = args.as_tuple().context("call with non-tuple value")?;

        (self.0)(args)
    }
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Callable")
            .field(&format!("<Callable @ {:016X?}>", Arc::as_ptr(&self.0)))
            .finish()
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Hash for Callable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
