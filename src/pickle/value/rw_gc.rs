use std::fmt;
use std::ops::Deref;

use dumpster::Trace;
use dumpster::sync::Gc;
use parking_lot::RwLock;

use crate::pickle::value::Id;
use crate::pickle::value::traced::Traced;

#[derive(Trace)]
pub struct RwGc<T>(Gc<Traced<RwLock<T>>>)
where
    T: Trace + Send + Sync + 'static;

impl<T> RwGc<T>
where
    T: Trace + Send + Sync + 'static,
{
    pub fn new(value: T) -> Self {
        Self(Gc::new(Traced(RwLock::new(value))))
    }

    pub fn id(&self) -> Id {
        Gc::as_ptr(&self.0).into()
    }
}

impl<T> Deref for RwGc<T>
where
    T: Trace + Send + Sync + 'static,
{
    type Target = RwLock<T>;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl<T> Clone for RwGc<T>
where
    T: Trace + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> fmt::Debug for RwGc<T>
where
    T: fmt::Debug + Trace + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Some(inner) = self.0.as_ref().try_read() else {
            return write!(f, "<Locked RwGc>");
        };

        inner.fmt(f)
    }
}
