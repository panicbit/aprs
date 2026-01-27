use std::fmt;
use std::sync::Arc;

use eyre::{Context, Result};

use crate::storage::{AsPtr, SameAs};
use crate::{Storage, Tuple};

use super::Value;

#[derive(Clone)]
#[expect(clippy::type_complexity)]
pub struct Callable<S: Storage>(Arc<dyn Fn(&Tuple<S>) -> Result<Value<S>> + Send + Sync>);

impl<S: Storage> Callable<S> {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&Tuple<S>) -> Result<Value<S>> + Send + Sync + 'static,
    {
        Self(Arc::new(f))
    }

    pub fn call(&self, args: Value<S>) -> Result<Value<S>> {
        let args = args.as_tuple().context("call with non-tuple value")?;

        (self.0)(args)
    }
}

impl<S: Storage> fmt::Debug for Callable<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Callable")
            .field(&format!("<Callable @ {:016X?}>", self.0.as_ptr()))
            .finish()
    }
}

impl<S: Storage> PartialEq for Callable<S> {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_as(&other.0)
    }
}
