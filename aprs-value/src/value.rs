use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, Mul, Sub};

use eyre::{Result, bail};
use tracing::error;

use crate::{Bool, Callable, Dict, Float, Int, List, Set, Storage, Str, Tuple};

pub enum Value<S: Storage> {
    Dict(Dict<S>),
    List(List<S>),
    Str(Str<S>),
    Int(Int),
    Float(Float),
    Bool(Bool),
    Tuple(Tuple<S>),
    Callable(Callable<S>),
    None(crate::None),
    Set(Set<S>),
}

impl<S: Storage> Value<S> {
    pub fn empty_dict() -> Self {
        Value::Dict(Dict::default())
    }

    pub fn empty_list() -> Self {
        Value::List(List::new())
    }

    pub fn bool(v: bool) -> Self {
        Self::Bool(v.into())
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
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Bool(_) => "Bool",
            Value::Tuple(_) => "Tuple",
            Value::Callable(_) => "Callable",
            Value::None(_) => "None",
            Value::Set(_) => "Set",
        }
    }

    pub fn extend(&self, value: Vec<Value<S>>) -> Result<()> {
        match self {
            Value::List(list) => {
                list.extend(value);
                Ok(())
            }
            _ => bail!("can't extend {}", self.type_name()),
        }
    }

    // TODO: move conversions to TryFrom impls

    pub fn as_dict(&self) -> Result<&Dict<S>> {
        match self {
            Value::Dict(dict) => Ok(dict),
            _ => bail!("{} is not a Dict", self.type_name()),
        }
    }

    pub fn as_list(&self) -> Result<&List<S>> {
        match self {
            Value::List(list) => Ok(list),
            _ => bail!("{} is not a List", self.type_name()),
        }
    }

    pub fn as_str(&self) -> Result<&Str<S>> {
        match self {
            Value::Str(str) => Ok(str),
            _ => bail!("{} is not a Str", self.type_name()),
        }
    }

    pub fn as_int(&self) -> Result<&Int> {
        match self {
            Value::Int(int) => Ok(int),
            _ => bail!("{} is not an Int", self.type_name()),
        }
    }

    pub fn as_float(&self) -> Result<&Float> {
        match self {
            Value::Float(float) => Ok(float),
            _ => bail!("{} is not a Float", self.type_name()),
        }
    }

    pub fn as_tuple(&self) -> Result<&Tuple<S>> {
        match self {
            Value::Tuple(tuple) => Ok(tuple),
            _ => bail!("{} is not a Tuple", self.type_name()),
        }
    }

    pub fn as_callable(&self) -> Result<&Callable<S>> {
        match self {
            Value::Callable(callable) => Ok(callable),
            _ => bail!("{} is not a Callable", self.type_name()),
        }
    }

    pub fn as_set(&self) -> Result<&Set<S>> {
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
            Value::Int(int) => int.hash(state),
            Value::Float(_) => bail!("Float is unhashable"),
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
            Value::Int(_) => true,
            Value::Float(_) => false,
            Value::Bool(_) => true,
            Value::Tuple(value) => value.is_hashable(),
            Value::Callable(_callable) => false,
            Value::None(_) => true,
            Value::Set(_) => false,
        }
    }

    pub fn tuple(value: impl Into<Tuple<S>>) -> Self {
        Self::Tuple(value.into())
    }

    pub fn empty_tuple() -> Self {
        Self::Tuple(Tuple::empty())
    }

    pub fn str(s: impl Into<String>) -> Self {
        Self::Str(Str::from(s.into()))
    }

    pub fn int(n: impl Into<Int>) -> Self {
        Self::Int(n.into())
    }

    pub fn float(n: impl Into<Float>) -> Self {
        Self::Float(n.into())
    }

    pub fn callable<F>(f: F) -> Self
    where
        F: Fn(&Tuple<S>) -> Result<Value<S>> + Send + Sync + 'static,
    {
        Self::Callable(Callable::new(f))
    }

    pub fn none() -> Self {
        Self::None(crate::None::new())
    }

    pub fn empty_set() -> Self {
        Self::Set(Set::new())
    }

    pub fn to_usize(&self) -> Option<usize> {
        match self {
            Value::Int(n) => n.to_usize(),
            _ => Option::None,
        }
    }
}

impl<S: Storage> Value<S> {
    pub fn add(&self, rhs: &Value<S>) -> Result<Value<S>> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Self::add_int(a, b),
            (Value::Float(a), Value::Float(b)) => Self::add_float(a, b),
            (Value::Int(a), Value::Float(b)) => Self::add_float(a, b),
            (Value::Float(a), Value::Int(b)) => Self::add_float(a, b),

            _ => bail!("Can't `add` {self:?} and {rhs:?}"),
        }
    }

    fn add_int(a: &Int, b: &Int) -> Result<Value<S>> {
        Ok(a.add(b).into())
    }

    fn add_float<A, B>(a: A, b: B) -> Result<Value<S>>
    where
        A: TryInto<Float>,
        eyre::Report: From<A::Error>,
        B: TryInto<Float>,
        eyre::Report: From<B::Error>,
    {
        let a = a.try_into()?;
        let b = b.try_into()?;

        Ok(a.add(b).into())
    }

    pub fn sub(&self, rhs: &Value<S>) -> Result<Value<S>> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Self::sub_int(a, b),
            (Value::Float(a), Value::Float(b)) => Self::sub_float(a, b),
            (Value::Int(a), Value::Float(b)) => Self::sub_float(a, b),
            (Value::Float(a), Value::Int(b)) => Self::sub_float(a, b),

            _ => bail!("Can't `sub` {self:?} and {rhs:?}"),
        }
    }

    fn sub_int(a: &Int, b: &Int) -> Result<Value<S>> {
        Ok(a.sub(b).into())
    }

    fn sub_float<A, B>(a: A, b: B) -> Result<Value<S>>
    where
        A: TryInto<Float>,
        eyre::Report: From<A::Error>,
        B: TryInto<Float>,
        eyre::Report: From<B::Error>,
    {
        let a = a.try_into()?;
        let b = b.try_into()?;

        Ok(a.sub(b).into())
    }

    pub fn mul(&self, rhs: &Value<S>) -> Result<Value<S>> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Self::mul_int(a, b),
            (Value::Float(a), Value::Float(b)) => Self::mul_float(a, b),
            (Value::Int(a), Value::Float(b)) => Self::mul_float(a, b),
            (Value::Float(a), Value::Int(b)) => Self::mul_float(a, b),

            _ => bail!("Can't `mul` {self:?} and {rhs:?}"),
        }
    }

    fn mul_int(a: &Int, b: &Int) -> Result<Value<S>> {
        Ok(a.mul(b).into())
    }

    fn mul_float<A, B>(a: A, b: B) -> Result<Value<S>>
    where
        A: TryInto<Float>,
        eyre::Report: From<A::Error>,
        B: TryInto<Float>,
        eyre::Report: From<B::Error>,
    {
        let a = a.try_into()?;
        let b = b.try_into()?;

        Ok(a.mul(b).into())
    }

    // fn pow(&self, rhs: &Value) -> Result<Value> {
    //     match (self, rhs) {
    //         (Value::Number(a), Value::Number(b)) => Ok(a.pow(b)).into()
    //         _ => bail!("Can't `pow` {self:?} and {rhs:?}"),
    //     }
    // }

    fn r#mod(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    pub fn floor(&self) -> Result<Value<S>> {
        match self {
            Self::Int(n) => Ok(Self::Int(n.clone())),
            Self::Float(n) => n.floor().map(Self::Int),
            _ => bail!("Can't `floor` {self:?}"),
        }
    }

    pub fn ceil(&self) -> Result<Value<S>> {
        match self {
            Self::Int(n) => Ok(Self::Int(n.clone())),
            Self::Float(n) => n.ceil().map(Self::Int),
            _ => bail!("Can't `ceil` {self:?}"),
        }
    }

    fn max(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    fn min(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    pub fn and(&self, rhs: &Value<S>) -> Result<Value<S>> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Self::Int(a.bitand(b))),
            (Value::Bool(a), Value::Bool(b)) => Ok((**a && **b).into()),
            _ => bail!("Can't `and` {self:?} and {rhs:?}"),
        }
    }

    pub fn or(&self, rhs: &Value<S>) -> Result<Value<S>> {
        match (self, rhs) {
            (Value::Int(a), Value::Int(b)) => Ok(Self::Int(a.bitor(b))),
            (Value::Bool(a), Value::Bool(b)) => Ok((**a || **b).into()),
            _ => bail!("Can't `or` {self:?} and {rhs:?}"),
        }
    }

    fn xor(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    fn left_shift(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    fn right_shift(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    fn remove(&self, _rhs: &Value<S>) -> Result<Value<S>> {
        todo!()
    }

    pub fn pop(&self, rhs: &Value<S>) -> Result<Option<Value<S>>> {
        let list = self.as_list()?;

        list.remove(rhs)
    }

    pub fn update(&self, dict: &Value<S>) -> Result<()> {
        let this = self.as_dict()?;
        let dict = dict.as_dict()?;

        this.update(dict)
    }
}

impl<S> Clone for Value<S>
where
    S: Storage,
{
    fn clone(&self) -> Self {
        match self {
            Self::Dict(arg0) => Self::Dict(arg0.clone()),
            Self::List(arg0) => Self::List(arg0.clone()),
            Self::Str(arg0) => Self::Str(arg0.clone()),
            Self::Int(arg0) => Self::Int(arg0.clone()),
            Self::Float(arg0) => Self::Float(arg0.clone()),
            Self::Bool(arg0) => Self::Bool(arg0.clone()),
            Self::Tuple(arg0) => Self::Tuple(arg0.clone()),
            Self::Callable(arg0) => Self::Callable(arg0.clone()),
            Self::None(arg0) => Self::None(arg0.clone()),
            Self::Set(arg0) => Self::Set(arg0.clone()),
        }
    }
}

impl<S: Storage> Default for Value<S> {
    fn default() -> Self {
        Value::none()
    }
}

impl<S: Storage> From<&Value<S>> for Value<S> {
    fn from(value: &Value<S>) -> Self {
        value.clone()
    }
}

impl<S: Storage> From<&str> for Value<S> {
    fn from(str: &str) -> Self {
        Self::str(str)
    }
}

impl<S: Storage> From<Dict<S>> for Value<S> {
    fn from(dict: Dict<S>) -> Self {
        Value::Dict(dict)
    }
}

impl<S: Storage> From<List<S>> for Value<S> {
    fn from(list: List<S>) -> Self {
        Value::List(list)
    }
}

impl<S: Storage> From<Str<S>> for Value<S> {
    fn from(str: Str<S>) -> Self {
        Value::Str(str)
    }
}

impl<S: Storage> From<Int> for Value<S> {
    fn from(n: Int) -> Self {
        Value::Int(n)
    }
}

impl<S: Storage> From<Float> for Value<S> {
    fn from(n: Float) -> Self {
        Value::Float(n)
    }
}

impl<S: Storage> From<i32> for Value<S> {
    fn from(value: i32) -> Self {
        Value::from(Int::from(value))
    }
}

impl<S: Storage> From<f64> for Value<S> {
    fn from(value: f64) -> Self {
        Value::from(Float::from(value))
    }
}

impl<S: Storage> From<Bool> for Value<S> {
    fn from(value: Bool) -> Self {
        Value::Bool(value)
    }
}

impl<S: Storage> From<bool> for Value<S> {
    fn from(value: bool) -> Self {
        Value::from(Bool::from(value))
    }
}

impl<S: Storage> fmt::Debug for Value<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Dict(dict) => f.debug_tuple("Dict").field(dict).finish(),
            Value::List(list) => f.debug_tuple("List").field(list).finish(),
            Value::Str(str) => str.fmt(f),
            Value::Int(int) => int.fmt(f),
            Value::Float(float) => float.fmt(f),
            Value::Bool(bool) => f.debug_tuple("Bool").field(bool).finish(),
            Value::Tuple(tuple) => tuple.fmt(f),
            Value::Callable(callable) => f.debug_tuple("Callable").field(&callable).finish(),
            Value::None(none) => f.debug_tuple("None").field(none).finish(),
            Value::Set(set) => f.debug_tuple("Set").field(set).finish(),
        }
    }
}

impl<S: Storage> PartialEq for Value<S> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Dict(v1) => match other {
                Value::Dict(v2) => v1 == v2,
                _ => false,
            },
            Value::List(v1) => match other {
                Value::List(v2) => v1 == v2,
                _ => false,
            },
            Value::Str(v1) => match other {
                Value::Str(v2) => v1.as_str() == v2.as_str(),
                _ => false,
            },
            Value::Int(v1) => match other {
                Value::Int(v2) => v1 == v2,
                Value::Float(v2) => v1 == v2,
                Value::Bool(v2) => v1 == v2,
                _ => false,
            },
            Value::Float(v1) => match other {
                Value::Int(v2) => v1 == v2,
                Value::Float(v2) => v1 == v2,
                Value::Bool(v2) => v1 == v2,
                _ => false,
            },
            Value::Bool(v1) => match other {
                Value::Int(v2) => v1 == v2,
                Value::Float(v2) => v1 == v2,
                Value::Bool(v2) => v1 == v2,
                _ => false,
            },
            Value::Tuple(v1) => match other {
                Value::Tuple(v2) => v1 == v2,
                _ => false,
            },
            Value::Callable(v1) => match other {
                Value::Callable(v2) => v1 == v2,
                _ => false,
            },
            Value::None(_) => matches!(other, Value::None(_)),
            Value::Set(v1) => match other {
                Value::Set(v2) => v1 == v2,
                _ => false,
            },
        }
    }
}

impl<S: Storage> Hash for Value<S> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
            .unwrap_or_else(|err| error!("hash of unhashable value: {err}"));
    }
}

// This is a lie. All bets are off.
impl<S: Storage> Eq for Value<S> {}
