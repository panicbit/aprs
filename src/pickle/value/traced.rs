use std::ops;

use dumpster::Trace;
use parking_lot::RwLock;

use crate::{FnvIndexMap, FnvIndexSet};

pub struct Traced<T>(pub T);

impl<T> ops::Deref for Traced<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for Traced<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl<T> Trace for Traced<RwLock<T>>
where
    T: Trace,
{
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        self.0.try_read().ok_or(())?.accept(visitor)
    }
}

unsafe impl<K, V> Trace for Traced<FnvIndexMap<K, V>>
where
    K: Trace,
    V: Trace,
{
    fn accept<VI: dumpster::Visitor>(&self, visitor: &mut VI) -> Result<(), ()> {
        for (k, v) in &self.0 {
            k.accept(visitor)?;
            v.accept(visitor)?;
        }

        Ok(())
    }
}

unsafe impl<K> Trace for Traced<FnvIndexSet<K>>
where
    K: Trace,
{
    fn accept<V: dumpster::Visitor>(&self, visitor: &mut V) -> Result<(), ()> {
        for k in &self.0 {
            k.accept(visitor)?;
        }

        Ok(())
    }
}
