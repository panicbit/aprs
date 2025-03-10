use std::hash::{Hash, Hasher};
use std::{fmt, ops};

use anyhow::{Result, bail};
use dumpster::Trace;
use dumpster::sync::Gc;

mod list;
pub use list::List;

mod dict;
pub use dict::Dict;

mod number_cache;
pub use number_cache::NumberCache;

mod number;
pub use number::Number;

mod tuple;
pub use tuple::Tuple;

mod callable;
pub use callable::Callable;

#[derive(Trace, Clone)]
pub enum Value {
    Dict(Gc<Dict>),
    List(Gc<List>),
    Str(Gc<Str>),
    Number(Gc<Number>),
    Bool(Gc<bool>),
    Tuple(Gc<Tuple>),
    Callable(Callable),
}

impl Value {
    pub fn id(&self) -> Id {
        match self {
            Value::Dict(gc) => gc.into(),
            Value::List(gc) => gc.into(),
            Value::Str(gc) => gc.into(),
            Value::Number(gc) => gc.into(),
            Value::Bool(gc) => gc.into(),
            Value::Tuple(gc) => gc.into(),
            Value::Callable(callable) => callable.id(),
        }
    }

    pub fn empty_dict() -> Self {
        Value::Dict(Gc::new(Dict::default()))
    }

    pub fn empty_list() -> Self {
        Value::List(Gc::new(List::new()))
    }

    pub fn extend(&self, value: impl Into<Value>) -> Result<()> {
        let value = value.into();

        match self {
            Value::List(list) => list.extend(value),
            Value::Dict(_) => bail!("can't extend Dict"),
            Value::Str(_) => bail!("can't extend Str"),
            Value::Number(_) => bail!("can't extend Number"),
            Value::Bool(_) => bail!("can't extend Bool"),
            Value::Tuple(_) => bail!("cant extend Tuple"),
            Value::Callable(_) => bail!("can't extend Callable"),
        }
    }

    // TODO: move conversions TryFrom impls

    pub fn as_dict(&self) -> Result<Gc<Dict>> {
        match self {
            Value::Dict(value) => Ok(value.clone()),
            Value::List(_) => bail!("List is not a Dict"),
            Value::Str(_) => bail!("Str is not a Dict"),
            Value::Number(_) => bail!("Number is not a Dict"),
            Value::Bool(_) => bail!("Bool is not a Dict"),
            Value::Tuple(_) => bail!("Tuple is not a Dict"),
            Value::Callable(_) => bail!("Callable is not a Dict"),
        }
    }

    pub fn as_str(&self) -> Result<Gc<Str>> {
        match self {
            Value::Dict(_) => bail!("Dict is not a Str"),
            Value::List(_) => bail!("List is not a Str"),
            Value::Str(value) => Ok(value.clone()),
            Value::Number(_) => bail!("Number is not a Str"),
            Value::Bool(_) => bail!("Bool is not a Str"),
            Value::Tuple(_) => bail!("Tuple is not a Str"),
            Value::Callable(_) => bail!("Callable is not a Str"),
        }
    }

    pub fn as_number(&self) -> Result<Gc<Number>> {
        match self {
            Value::Dict(_) => bail!("Dict is not a Tuple"),
            Value::List(_) => bail!("List is not a Tuple"),
            Value::Str(_) => bail!("Str is not a Tuple"),
            Value::Number(value) => Ok(value.clone()),
            Value::Bool(_) => bail!("Bool is not a Tuple"),
            Value::Tuple(_) => bail!("Tuple is not a Number"),
            Value::Callable(_) => bail!("Callable is not a Tuple"),
        }
    }

    pub fn as_tuple(&self) -> Result<Gc<Tuple>> {
        match self {
            Value::Dict(_) => bail!("Dict is not a Tuple"),
            Value::List(_) => bail!("List is not a Tuple"),
            Value::Str(_) => bail!("Str is not a Tuple"),
            Value::Number(_) => bail!("Number is not a Tuple"),
            Value::Bool(_) => bail!("Bool is not a Tuple"),
            Value::Tuple(value) => Ok(value.clone()),
            Value::Callable(_) => bail!("Callable is not a Tuple"),
        }
    }

    pub fn as_callable(&self) -> Result<&Callable> {
        match self {
            Value::Dict(_) => bail!("Dict is not a Callable"),
            Value::List(_) => bail!("List is not a Callable"),
            Value::Str(_) => bail!("Str is not a Callable"),
            Value::Number(_) => bail!("Number is not a Callable"),
            Value::Bool(_) => bail!("Bool is not a Callable"),
            Value::Tuple(_) => bail!("Tuple is not a Callable"),
            Value::Callable(callable) => Ok(callable),
        }
    }

    pub fn hash<H: Hasher>(&self, state: &mut H) -> Result<()> {
        match self {
            Value::Dict(_) => bail!("Dict is unhashable"),
            Value::List(_) => bail!("List is unhashable"),
            Value::Str(gc) => gc.as_ref().hash(state),
            Value::Number(gc) => gc.as_ref().hash(state),
            Value::Bool(gc) => gc.as_ref().hash(state),
            Value::Tuple(gc) => gc.as_ref().hash(state),
            Value::Callable(_callable) => bail!("Callable is unhashable"),
        }

        Ok(())
    }

    pub fn is_hashable(&self) -> bool {
        match self {
            Value::Dict(_) => false,
            Value::List(_) => false,
            Value::Str(_) => true,
            Value::Number(_) => true,
            Value::Bool(_) => true,
            Value::Tuple(value) => value.is_hashable(),
            Value::Callable(_callable) => false,
        }
    }

    pub fn tuple(value: impl Into<Tuple>) -> Self {
        Self::Tuple(Gc::new(value.into()))
    }

    pub fn empty_tuple() -> Self {
        Self::Tuple(Gc::new(Tuple::empty()))
    }

    pub fn str(s: impl Into<String>) -> Self {
        let s = s.into();

        Self::Str(Gc::new(Str(s)))
    }

    pub fn callable<F>(f: F) -> Self
    where
        F: Fn(&Tuple) -> Result<Value> + Send + Sync + 'static,
    {
        Self::Callable(Callable::new(f))
    }
}

impl From<Dict> for Value {
    fn from(value: Dict) -> Self {
        Value::Dict(Gc::new(value))
    }
}

impl From<Gc<List>> for Value {
    fn from(value: Gc<List>) -> Self {
        Value::List(value)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Dict(gc) => f.debug_tuple("Map").field(gc.as_ref()).finish(),
            Value::List(gc) => f.debug_tuple("List").field(gc.as_ref()).finish(),
            Value::Str(gc) => f.debug_tuple("Str").field(gc.as_ref()).finish(),
            Value::Number(gc) => f.debug_tuple("Number").field(gc.as_ref()).finish(),
            Value::Bool(gc) => f.debug_tuple("Bool").field(gc.as_ref()).finish(),
            Value::Tuple(gc) => f.debug_tuple("Tuple").field(gc.as_ref()).finish(),
            Value::Callable(callable) => f.debug_tuple("Callable").field(&callable).finish(),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Dict(v1), Value::Dict(v2)) => v1 == v2,
            (Value::Dict(_), _) => false,
            (_, Value::Dict(_)) => false,
            (Value::List(v1), Value::List(v2)) => v1 == v2,
            (Value::List(_), _) => false,
            (_, Value::List(_)) => false,
            (Value::Str(v1), Value::Str(v2)) => v1 == v2,
            (Value::Str(_), _) => false,
            (_, Value::Str(_)) => false,
            (Value::Number(v1), Value::Number(v2)) => v1 == v2,
            (Value::Bool(v1), Value::Bool(v2)) => v1 == v2,
            (Value::Number(v1), Value::Bool(v2)) => v1.as_ref() == &Number::from(*v2.as_ref()),
            (Value::Bool(v1), Value::Number(v2)) => v2.as_ref() == &Number::from(*v1.as_ref()),
            (Value::Tuple(v1), Value::Tuple(v2)) => v1 == v2,
            (Value::Tuple(_), _) => false,
            (_, Value::Tuple(_)) => false,
            (Value::Callable(v1), Value::Callable(v2)) => v1 == v2,
            (Value::Callable(_), _) => false,
            (_, Value::Callable(_)) => false,
        }
    }
}

#[derive(Trace, Default, PartialEq, Hash)]
pub struct Str(String);

impl Str {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for Str {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<&'_ str> for Str {
    fn from(value: &'_ str) -> Self {
        Self(String::from(value))
    }
}

impl ops::Deref for Str {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(*const ());

unsafe impl Send for Id {}
unsafe impl Sync for Id {}

impl<T: Trace + Send + Sync> From<&'_ Gc<T>> for Id {
    fn from(value: &Gc<T>) -> Self {
        Self::from(Gc::as_ptr(value))
    }
}

impl<T: Trace + Send + Sync> From<Gc<T>> for Id {
    fn from(value: Gc<T>) -> Self {
        Self::from(Gc::as_ptr(&value))
    }
}

impl<T: ?Sized> From<*const T> for Id {
    fn from(ptr: *const T) -> Self {
        Self(ptr.cast::<()>())
    }
}

impl fmt::Debug for Str {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
