use std::hash::{Hash, Hasher};

use dumpster::Trace;

use crate::pickle::value::{List, Value};

#[derive(Trace, Debug, PartialEq)]
pub struct Tuple(List);

impl Tuple {
    pub fn is_hashable(&self) -> bool {
        self.0.iter().all(|value| value.is_hashable())
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

impl Hash for Tuple {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for value in &self.0 {
            value.hash(state);
        }
    }
}
