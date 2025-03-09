use std::fmt;
use std::hash::{Hash, Hasher};

use anyhow::{Result, bail};
use bstr::ByteSlice;
use dumpster::Trace;
use dumpster::sync::Gc;

mod list;
pub use list::List;

mod dict;
pub use dict::Dict;

mod big_int;
pub use big_int::BigInt;

mod number_cache;
pub use number_cache::NumberCache;

#[derive(Trace, Clone)]
pub enum Value {
    Dict(Gc<Dict>),
    List(Gc<List>),
    BinStr(Gc<BinStr>),
    Byte(Gc<u8>),
    Int(Gc<i32>),
    BigInt(Gc<BigInt>),
}

impl Value {
    pub fn id(&self) -> Id {
        match self {
            Value::Dict(gc) => gc.into(),
            Value::List(gc) => gc.into(),
            Value::BinStr(gc) => gc.into(),
            Value::Byte(gc) => gc.into(),
            Value::Int(gc) => gc.into(),
            Value::BigInt(gc) => gc.into(),
        }
    }

    pub fn empty_dict() -> Self {
        Value::Dict(Gc::new(Dict::default()))
    }

    pub fn empty_list() -> Self {
        Value::List(List::new())
    }

    pub fn append(&self, value: impl Into<Value>) -> Result<()> {
        let value = value.into();

        match self {
            Value::List(list) => list.append(value),
            Value::Dict(_) => bail!("can't append to Dict"),
            Value::BinStr(_) => bail!("can't append to BinStr"),
            Value::Byte(_) => bail!("can't append to Byte"),
            Value::Int(_) => bail!("can't append to Int"),
            Value::BigInt(_) => bail!("can't append to BigInt"),
        }
    }

    pub fn as_dict(&self) -> Result<Gc<Dict>> {
        match self {
            Value::Dict(value) => Ok(value.clone()),
            Value::List(_) => bail!("List is not a Dict"),
            Value::BinStr(_) => bail!("BinStr is not a Dict"),
            Value::Byte(_) => bail!("Byte is not a Dict"),
            Value::Int(_) => bail!("Int is not a Dict"),
            Value::BigInt(_) => bail!("BigInt is not a Dict"),
        }
    }

    pub fn hash<H: Hasher>(&self, state: &mut H) -> Result<()> {
        match self {
            Value::Dict(_) => bail!("Dict is unhashable"),
            Value::List(_) => bail!("List is unhashable"),
            Value::BinStr(gc) => gc.as_ref().hash(state),
            Value::Byte(gc) => gc.as_ref().hash(state),
            Value::Int(gc) => gc.as_ref().hash(state),
            Value::BigInt(gc) => gc.as_ref().hash(state),
        }

        Ok(())
    }

    pub fn is_hashable(&self) -> bool {
        match self {
            Value::Dict(_) => false,
            Value::List(_) => false,
            Value::BinStr(_) => true,
            Value::Byte(_) => true,
            Value::Int(_) => true,
            Value::BigInt(_) => true,
        }
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
            Value::Dict(_) => todo!("implement Debug for Dict"),
            Value::List(value) => f.debug_tuple("List").field(&**value).finish(),
            Value::BinStr(value) => f.debug_tuple("BinStr").field(&**value).finish(),
            Value::Byte(value) => f.debug_tuple("Byte").field(&**value).finish(),
            Value::Int(value) => f.debug_tuple("Int").field(&**value).finish(),
            Value::BigInt(value) => f.debug_tuple("BigInt").field(&**value).finish(),
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
            (Value::BinStr(v1), Value::BinStr(v2)) => v1 == v2,
            (Value::BinStr(_), _) => false,
            (_, Value::BinStr(_)) => false,
            (Value::Byte(v1), Value::Byte(v2)) => v1 == v2,
            (Value::Byte(v1), Value::Int(v2)) => i32::from(*v1.as_ref()) == *v2.as_ref(),
            (Value::Byte(v1), Value::BigInt(v2)) => BigInt::from(*v1.as_ref()) == *v2.as_ref(),
            (Value::Int(v1), Value::Byte(v2)) => *v1.as_ref() == i32::from(*v2.as_ref()),
            (Value::Int(v1), Value::Int(v2)) => v1 == v2,
            (Value::Int(v1), Value::BigInt(v2)) => BigInt::from(*v1.as_ref()) == *v2.as_ref(),
            (Value::BigInt(v1), Value::Byte(v2)) => *v1.as_ref() == BigInt::from(*v2.as_ref()),
            (Value::BigInt(v1), Value::Int(v2)) => *v1.as_ref() == BigInt::from(*v2.as_ref()),
            (Value::BigInt(v1), Value::BigInt(v2)) => v1 == v2,
        }
    }
}

#[derive(Trace, Default, PartialEq, Hash)]
pub struct BinStr(pub Vec<u8>);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(*const ());

unsafe impl Send for Id {}
unsafe impl Sync for Id {}

impl<T: Trace + Send + Sync> From<&'_ Gc<T>> for Id {
    fn from(value: &Gc<T>) -> Self {
        let ptr = Gc::as_ptr(value).cast::<()>();

        Self(ptr)
    }
}

impl<T: Trace + Send + Sync> From<Gc<T>> for Id {
    fn from(value: Gc<T>) -> Self {
        Self::from(&value)
    }
}

impl fmt::Debug for BinStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.as_bstr().fmt(f)
    }
}
