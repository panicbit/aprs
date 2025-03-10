use std::sync::Arc;
use std::{fmt, ptr};

use anyhow::{Context, Result};
use dumpster::Trace;

use crate::pickle::value::{Id, Tuple};

use super::Value;

/// Do not store GC references in a Callable or they will leak.
/// It's currently not possible to implement the `Trace` properly for it.
#[derive(Clone)]
#[allow(clippy::type_complexity)]
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

    pub fn id(&self) -> Id {
        Id::from(Arc::as_ptr(&self.0))
    }
}

unsafe impl Trace for Callable {
    fn accept<V: dumpster::Visitor>(&self, _visitor: &mut V) -> Result<(), ()> {
        Ok(())
    }
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Callable")
            .field(&format!("<Callable @ 0x{:016X?}>", self.id().0))
            .finish()
    }
}

impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(Arc::as_ptr(&self.0), Arc::as_ptr(&other.0))
    }
}
