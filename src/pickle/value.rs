use std::fmt;
use std::hash::{Hash, Hasher};

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

mod none;
pub use none::None;

mod set;
pub use set::Set;

mod str;
pub use str::Str;

mod bool;
pub use bool::Bool;

mod rw_gc;
mod traced;

#[derive(Trace, Clone)]
pub enum Value {
    Dict(Dict),
    List(List),
    Str(Str),
    Number(Number),
    Bool(Bool),
    Tuple(Tuple),
    Callable(Callable),
    None(None),
    Set(Set),
}

impl Value {
    pub fn id(&self) -> Id {
        match self {
            Value::Dict(dict) => dict.id(),
            Value::List(list) => list.id(),
            Value::Str(str) => str.id(),
            Value::Number(number) => number.id(),
            Value::Bool(bool) => bool.id(),
            Value::Tuple(tuple) => tuple.id(),
            Value::Callable(callable) => callable.id(),
            Value::None(none) => none.id(),
            Value::Set(set) => set.id(),
        }
    }

    pub fn empty_dict() -> Self {
        Value::Dict(Dict::default())
    }

    pub fn empty_list() -> Self {
        Value::List(List::new())
    }

    #[expect(non_snake_case)]
    pub fn False() -> Self {
        Self::Bool(Bool::False())
    }

    #[expect(non_snake_case)]
    pub fn True() -> Self {
        Self::Bool(Bool::True())
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Dict(_) => "Dict",
            Value::List(_) => "List",
            Value::Str(_) => "Str",
            Value::Number(_) => "Number",
            Value::Bool(_) => "Bool",
            Value::Tuple(_) => "Tuple",
            Value::Callable(_) => "Callable",
            Value::None(_) => "None",
            Value::Set(_) => "Set",
        }
    }

    pub fn extend(&self, value: impl Into<Value>) -> Result<()> {
        let value = value.into();

        match self {
            Value::List(list) => list.extend(value),
            _ => bail!("can't extend {}", self.type_name()),
        }
    }

    // TODO: move conversions TryFrom impls

    pub fn as_dict(&self) -> Result<Dict> {
        match self {
            Value::Dict(dict) => Ok(dict.clone()),
            _ => bail!("{} is not a Dict", self.type_name()),
        }
    }

    pub fn as_list(&self) -> Result<List> {
        match self {
            Value::List(list) => Ok(list.clone()),
            _ => bail!("{} is not a List", self.type_name()),
        }
    }

    pub fn as_str(&self) -> Result<Str> {
        match self {
            Value::Str(str) => Ok(str.clone()),
            _ => bail!("{} is not a Str", self.type_name()),
        }
    }

    pub fn as_number(&self) -> Result<Number> {
        match self {
            Value::Number(number) => Ok(number.clone()),
            _ => bail!("{} is not a Number", self.type_name()),
        }
    }

    pub fn as_tuple(&self) -> Result<Tuple> {
        match self {
            Value::Tuple(tuple) => Ok(tuple.clone()),
            _ => bail!("{} is not a Tuple", self.type_name()),
        }
    }

    pub fn as_callable(&self) -> Result<Callable> {
        match self {
            Value::Callable(callable) => Ok(callable.clone()),
            _ => bail!("{} is not a Callable", self.type_name()),
        }
    }

    pub fn as_set(&self) -> Result<&Set> {
        match self {
            Value::Set(set) => Ok(set),
            _ => bail!("{} is not a Set", self.type_name()),
        }
    }

    pub fn hash<H: Hasher>(&self, state: &mut H) -> Result<()> {
        match self {
            Value::Dict(_) => bail!("Dict is unhashable"),
            Value::List(_) => bail!("List is unhashable"),
            Value::Str(str) => str.hash(state),
            Value::Number(number) => number.hash(state),
            Value::Bool(bool) => bool.hash(state),
            Value::Tuple(tuple) => tuple.hash(state),
            Value::Callable(_callable) => bail!("Callable is unhashable"),
            Value::None(none) => none.hash(state),
            Value::Set(_) => bail!("Set is unhashable"),
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
            Value::None(_) => true,
            Value::Set(_) => false,
        }
    }

    pub fn tuple(value: impl Into<Tuple>) -> Self {
        Self::Tuple(value.into())
    }

    pub fn empty_tuple() -> Self {
        Self::Tuple(Tuple::empty())
    }

    pub fn str(s: impl Into<String>) -> Self {
        Self::Str(Str::from(s.into()))
    }

    pub fn callable<F>(f: F) -> Self
    where
        F: Fn(&Tuple) -> Result<Value> + Send + Sync + 'static,
    {
        Self::Callable(Callable::new(f))
    }

    pub fn none() -> Self {
        Self::None(None::new())
    }

    pub fn empty_set() -> Self {
        Self::Set(Set::new())
    }
}

impl From<&str> for Value {
    fn from(str: &str) -> Self {
        Self::str(str)
    }
}

impl From<Dict> for Value {
    fn from(dict: Dict) -> Self {
        Value::Dict(dict)
    }
}

impl From<List> for Value {
    fn from(list: List) -> Self {
        Value::List(list)
    }
}

impl From<Str> for Value {
    fn from(str: Str) -> Self {
        Value::Str(str)
    }
}

impl From<Number> for Value {
    fn from(number: Number) -> Self {
        Value::Number(number)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Dict(dict) => f.debug_tuple("Dict").field(dict).finish(),
            Value::List(list) => f.debug_tuple("List").field(list).finish(),
            Value::Str(str) => f.debug_tuple("Str").field(str).finish(),
            Value::Number(number) => f.debug_tuple("Number").field(number).finish(),
            Value::Bool(bool) => f.debug_tuple("Bool").field(bool).finish(),
            Value::Tuple(tuple) => f.debug_tuple("Tuple").field(tuple).finish(),
            Value::Callable(callable) => f.debug_tuple("Callable").field(&callable).finish(),
            Value::None(none) => f.debug_tuple("None").field(none).finish(),
            Value::Set(set) => f.debug_tuple("Set").field(set).finish(),
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
            (Value::Number(v1), Value::Bool(v2)) => v1 == &Number::from(**v2),
            (Value::Bool(v1), Value::Number(v2)) => v2 == &Number::from(**v1),
            (Value::Tuple(v1), Value::Tuple(v2)) => v1 == v2,
            (Value::Tuple(_), _) => false,
            (_, Value::Tuple(_)) => false,
            (Value::Callable(v1), Value::Callable(v2)) => v1 == v2,
            (Value::Callable(_), _) => false,
            (_, Value::Callable(_)) => false,
            (Value::None(_), Value::None(_)) => true,
            (Value::None(_), _) => false,
            (_, Value::None(_)) => false,
            (Value::Set(v1), Value::Set(v2)) => v1 == v2,
            (Value::Set(_), _) => false,
            (_, Value::Set(_)) => false,
        }
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
