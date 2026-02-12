use aprs_value::Value;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "o")]
pub struct Set {
    pub key: String,
    pub default: Value,
    #[serde(default = "bool_true")]
    pub want_reply: bool,
    pub operations: Vec<SetOperation>,
}

fn bool_true() -> bool {
    true
}

// TODO: Maybe restrict the value range for binary operations (e.g. disallow strings for math ops).
//       Probably pointless due to the existence of the `default` field in `Set`.

#[derive(Serialize, Deserialize, Debug)]
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
