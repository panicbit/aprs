use serde::Deserialize;

use crate::pickle::Value;
use crate::pickle::value::{Str, storage};

type S = storage::Arc;

#[derive(Deserialize, Debug)]
#[serde(tag = "o")]
pub struct Set {
    pub key: Str<S>,
    pub default: Value<S>,
    #[serde(default = "bool_true")]
    pub want_reply: bool,
    pub operations: Vec<SetOperation>,
}

fn bool_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
#[serde(tag = "operation", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum SetOperation {
    Default,
    Replace(Value<S>),
    Add(Value<S>),
    Mul(Value<S>),
    Pow(Value<S>),
    Mod(Value<S>),
    Floor,
    Ceil,
    Max(Value<S>),
    Min(Value<S>),
    And(Value<S>),
    Or(Value<S>),
    Xor(Value<S>),
    LeftShift(Value<S>),
    RightShift(Value<S>),
    Remove(Value<S>),
    Pop(Value<S>),
    Update(Value<S>),
}
