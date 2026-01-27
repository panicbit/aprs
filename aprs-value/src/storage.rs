use std::cell::{Ref, RefCell, RefMut};
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync;
use std::{ptr, rc};

use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait Storage: 'static + Clone + PartialEq + Eq + PartialOrd + Ord + Hash + Debug {
    type ReadOnly<T>: Deref<Target = T> + SameAs + AsPtr + Clone;
    type ReadWrite<T>: SameAs + AsPtr + Clone;
    type Read<'a, T: 'a>: Deref<Target = T>;
    type Write<'a, T: 'a>: DerefMut<Target = T>;

    fn new_read_only<T>(value: T) -> Self::ReadOnly<T>;
    fn new_read_write<T>(value: T) -> Self::ReadWrite<T>;
    fn read<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Read<'a, T>;
    fn write<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Write<'a, T>;
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Rc(());

impl Storage for Rc {
    type ReadOnly<T> = rc::Rc<T>;
    type ReadWrite<T> = rc::Rc<RefCell<T>>;
    type Read<'a, T: 'a> = Ref<'a, T>;
    type Write<'a, T: 'a> = RefMut<'a, T>;

    fn new_read_only<T>(value: T) -> Self::ReadOnly<T> {
        rc::Rc::new(value)
    }

    fn new_read_write<T>(value: T) -> Self::ReadWrite<T> {
        rc::Rc::new(RefCell::new(value))
    }

    fn read<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Read<'a, T> {
        rw.as_ref().borrow()
    }

    fn write<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Write<'a, T> {
        rw.as_ref().borrow_mut()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Arc();

impl Storage for Arc {
    type ReadOnly<T> = sync::Arc<T>;
    type ReadWrite<T> = sync::Arc<RwLock<T>>;
    type Read<'a, T: 'a> = RwLockReadGuard<'a, T>;
    type Write<'a, T: 'a> = RwLockWriteGuard<'a, T>;

    fn new_read_only<T>(value: T) -> Self::ReadOnly<T> {
        sync::Arc::new(value)
    }

    fn new_read_write<T>(value: T) -> Self::ReadWrite<T> {
        sync::Arc::new(RwLock::new(value))
    }

    fn read<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Read<'a, T> {
        rw.as_ref().read()
    }

    fn write<'a, T>(rw: &'a Self::ReadWrite<T>) -> Self::Write<'a, T> {
        rw.as_ref().write()
    }
}

pub trait SameAs {
    fn same_as(&self, other: &Self) -> bool;
}

impl<T: ?Sized> SameAs for rc::Rc<T> {
    fn same_as(&self, other: &Self) -> bool {
        let a = rc::Rc::as_ptr(self);
        let b = rc::Rc::as_ptr(other);

        ptr::addr_eq(a, b)
    }
}

impl<T: ?Sized> SameAs for sync::Arc<T> {
    fn same_as(&self, other: &Self) -> bool {
        let a = sync::Arc::as_ptr(self);
        let b = sync::Arc::as_ptr(other);

        ptr::addr_eq(a, b)
    }
}

pub trait AsPtr {
    fn as_ptr(&self) -> *const ();
}

impl<T: ?Sized> AsPtr for rc::Rc<T> {
    fn as_ptr(&self) -> *const () {
        rc::Rc::as_ptr(self).cast::<()>()
    }
}

impl<T: ?Sized> AsPtr for sync::Arc<T> {
    fn as_ptr(&self) -> *const () {
        sync::Arc::as_ptr(self).cast::<()>()
    }
}
