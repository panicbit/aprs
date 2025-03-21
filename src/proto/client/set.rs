use serde::Deserialize;

use crate::pickle::Value;
use crate::pickle::value::Str;

#[derive(Deserialize, Debug)]
#[serde(tag = "o")]
pub struct Set {
    pub key: Str,
    pub default: Value,
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
    Replace(Value),
    Add(Value),
    Mul(Value),
    Pow(Value),
    Mod(Value),
    Floor,
    Ceil,
    Max(Value),
    Min(Value),
    And(Value),
    Or(Value),
    Xor(Value),
    LeftShift(Value),
    RightShift(Value),
    Remove(Value),
    Pop(Value),
    Update(Value),
}
