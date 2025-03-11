use std::hash::{Hash, Hasher};

use anyhow::{Context, Error, Result, bail};
use dumpster::Trace;

use crate::pickle::value::{Id, List, Value};

// TODO: replace List with Vec. Tuples are immutable, so the underlying lock is not needed.

#[derive(Trace, Debug, PartialEq, Clone)]

pub struct Tuple(List);

impl Tuple {
    pub fn id(&self) -> Id {
        self.0.id()
    }

    pub fn empty() -> Self {
        Self(List::new())
    }

    pub fn is_hashable(&self) -> bool {
        self.0.iter().all(|value| value.is_hashable())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> Option<Value> {
        self.0.get(index)
    }
}

impl From<&List> for Tuple {
    fn from(list: &List) -> Self {
        list.iter().collect::<Tuple>()
    }
}

impl From<List> for Tuple {
    fn from(list: List) -> Self {
        list.iter().collect::<Tuple>()
    }
}

impl From<()> for Tuple {
    fn from(_: ()) -> Self {
        let tuple = List::new();

        Self(tuple)
    }
}

impl From<(Value,)> for Tuple {
    fn from((v1,): (Value,)) -> Self {
        let tuple = List::new();

        tuple.push(v1);

        Self(tuple)
    }
}

impl From<(Value, Value)> for Tuple {
    fn from((v1, v2): (Value, Value)) -> Self {
        let tuple = List::new();

        tuple.push(v1);
        tuple.push(v2);

        Self(tuple)
    }
}

impl From<(Value, Value, Value)> for Tuple {
    fn from((v1, v2, v3): (Value, Value, Value)) -> Self {
        let tuple = List::new();

        tuple.push(v1);
        tuple.push(v2);
        tuple.push(v3);

        Self(tuple)
    }
}

impl<V> FromIterator<V> for Tuple
where
    V: Into<Value>,
{
    fn from_iter<I: IntoIterator<Item = V>>(iter: I) -> Self {
        let list = iter.into_iter().collect::<List>();

        Self(list)
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

        let v1 = tuple.get(0).context("BUG: tuple too short")?;
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

        let v1 = tuple.get(0).context("BUG: tuple too short")?;
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?;
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

        let v1 = tuple.get(0).context("BUG: tuple too short")?;
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?;
        let v2 = V2::try_from(v2)?;
        let v3 = tuple.get(2).context("BUG: tuple too short")?;
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
            bail!("expected tuple of length 3");
        }

        let v1 = tuple.get(0).context("BUG: tuple too short")?;
        let v1 = V1::try_from(v1)?;
        let v2 = tuple.get(1).context("BUG: tuple too short")?;
        let v2 = V2::try_from(v2)?;
        let v3 = tuple.get(2).context("BUG: tuple too short")?;
        let v3 = V3::try_from(v3)?;
        let v4 = tuple.get(3).context("BUG: tuple too short")?;
        let v4 = V4::try_from(v4)?;

        Ok((v1, v2, v3, v4))
    }
}
