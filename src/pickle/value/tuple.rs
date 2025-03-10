use std::hash::{Hash, Hasher};

use anyhow::{Context, Error, Result, bail};
use dumpster::Trace;

use crate::pickle::value::{List, Value};

#[derive(Trace, Debug, PartialEq)]
pub struct Tuple(List);

impl Tuple {
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
