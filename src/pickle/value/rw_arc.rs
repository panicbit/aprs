use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::pickle::value::Id;

pub struct RwArc<T>(Arc<RwLock<T>>)
where
    T: Send + Sync + 'static;

impl<T> RwArc<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(value: T) -> Self {
        Self(Arc::new(RwLock::new(value)))
    }

    pub fn id(&self) -> Id {
        Arc::as_ptr(&self.0).into()
    }
}

impl<T> Deref for RwArc<T>
where
    T: Send + Sync + 'static,
{
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Clone for RwArc<T>
where
    T: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for RwArc<T>
where
    T: fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(inner) = self.0.as_ref().try_read() else {
            return write!(f, "<Locked RwGc>");
        };

        inner.fmt(f)
    }
}
