use std::sync::Arc;
use std::{fmt, ptr};

use eyre::{Context, Result};

use crate::pickle::value::Tuple;

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

        (self.0)(&args)
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
        ptr::addr_eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }
}
