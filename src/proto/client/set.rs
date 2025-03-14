use serde::Deserialize;

use crate::pickle::Value;

#[derive(Deserialize, Debug)]
#[serde(tag = "o")]
pub struct Set {
    pub key: String,
    pub default: Value,
    pub want_reply: bool,
    pub operations: Vec<SetOperation>,
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
