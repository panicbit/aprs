use std::hash::{Hash, Hasher};
use std::{fmt, slice};

use eyre::{ContextCompat, Error, Result, bail};

use crate::pickle::value::{List, Value};

type Iter<'a> = slice::Iter<'a, Value>;

#[derive(PartialEq, Clone)]

pub struct Tuple(Vec<Value>);

impl Tuple {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn is_hashable(&self) -> bool {
        self.0.iter().all(|value| value.is_hashable())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter {
        self.into_iter()
    }
}

impl From<&List> for Tuple {
    fn from(list: &List) -> Self {
        list.read().iter().collect::<Tuple>()
    }
}

impl From<List> for Tuple {
    fn from(list: List) -> Self {
        list.read().iter().collect::<Tuple>()
    }
}

impl From<()> for Tuple {
    fn from(_: ()) -> Self {
        let tuple = Vec::new();

        Self(tuple)
    }
}

impl From<(Value,)> for Tuple {
    fn from((v1,): (Value,)) -> Self {
        Self(vec![v1])
    }
}

impl From<(Value, Value)> for Tuple {
    fn from((v1, v2): (Value, Value)) -> Self {
        Self(vec![v1, v2])
    }
}

impl From<(Value, Value, Value)> for Tuple {
    fn from((v1, v2, v3): (Value, Value, Value)) -> Self {
        Self(vec![v1, v2, v3])
    }
}

impl<V> FromIterator<V> for Tuple
where
    V: Into<Value>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let tuple = iter.into_iter().map(<_>::into).collect::<Vec<_>>();

        Self(tuple)
    }
}

impl Hash for Tuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in &self.0 {
            value.hash(state).ok();
        }
    }
}

impl<V1> TryFrom<&'_ Tuple> for (V1,)
where
    V1: TryFrom<Value>,
    Error: From<V1::Error>,
{
    type Error = Error;

    fn try_from(tuple: &Tuple) -> Result<Self> {
        if tuple.len() != 1 {
            bail!("expected tuple of length 1");
        }

        let v1 = tuple.get(0).context("BUG: tuple too short")?.clone();
        let v1 = V1::try_from(v1)?;

        Ok((v1,))
    }
}

impl<V1, V2> TryFrom<&'_ Tuple> for (V1, V2)
where
    V1: TryFrom<Value>,
    V2: TryFrom<Value>,
    Error: From<V1::Error> + From<V2::Error>,
{
    type Error = Error;

    fn try_from(tuple: &Tuple) -> Result<Self> {
        if tuple.len() != 2 {
            bail!("expected tuple of length 2");
        }

        let v1 = tuple.get(0).context("BUG: tuple too short")?.clone();
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?.clone();
        let v2 = V2::try_from(v2)?;

        Ok((v1, v2))
    }
}

impl<V1, V2, V3> TryFrom<&'_ Tuple> for (V1, V2, V3)
where
    V1: TryFrom<Value>,
    V2: TryFrom<Value>,
    V3: TryFrom<Value>,
    Error: From<V1::Error> + From<V2::Error> + From<V3::Error>,
{
    type Error = Error;

    fn try_from(tuple: &Tuple) -> Result<Self> {
        if tuple.len() != 3 {
            bail!("expected tuple of length 3");
        }

        let v1 = tuple.get(0).context("BUG: tuple too short")?.clone();
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?.clone();
        let v2 = V2::try_from(v2)?;
        let v3 = tuple.get(2).context("BUG: tuple too short")?.clone();
        let v3 = V3::try_from(v3)?;

        Ok((v1, v2, v3))
    }
}

impl<V1, V2, V3, V4> TryFrom<&'_ Tuple> for (V1, V2, V3, V4)
where
    V1: TryFrom<Value>,
    V2: TryFrom<Value>,
    V3: TryFrom<Value>,
    V4: TryFrom<Value>,
    Error: From<V1::Error> + From<V2::Error> + From<V3::Error> + From<V4::Error>,
{
    type Error = Error;

    fn try_from(tuple: &Tuple) -> Result<Self> {
        if tuple.len() != 4 {
            bail!("expected tuple of length 4");
        }

        let v1 = tuple.get(0).context("BUG: tuple too short")?.clone();
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?.clone();
        let v2 = V2::try_from(v2)?;
        let v3 = tuple.get(2).context("BUG: tuple too short")?.clone();
        let v3 = V3::try_from(v3)?;
        let v4 = tuple.get(3).context("BUG: tuple too short")?.clone();
        let v4 = V4::try_from(v4)?;

        Ok((v1, v2, v3, v4))
    }
}

impl<'a> IntoIterator for &'a Tuple {
    type Item = &'a Value;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl fmt::Debug for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tuple = f.debug_tuple("");

        for value in self {
            tuple.field(&value);
        }

        tuple.finish()
    }
}
