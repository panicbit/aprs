use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "o")]
pub struct Set<V> {
    pub key: String,
    pub default: V,
    #[serde(default = "bool_true")]
    pub want_reply: bool,
    pub operations: Vec<SetOperation<V>>,
}

fn bool_true() -> bool {
    true
}

// TODO: Maybe restrict the value range for binary operations (e.g. disallow strings for math ops).
//       Probably pointless due to the existence of the `default` field in `Set`.

#[derive(Deserialize, Debug)]
#[serde(tag = "operation", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum SetOperation<V> {
    Default,
    Replace(V),
    Add(V),
    Mul(V),
    Pow(V),
    Mod(V),
    Floor,
    Ceil,
    Max(V),
    Min(V),
    And(V),
    Or(V),
    Xor(V),
    LeftShift(V),
    RightShift(V),
    Remove(V),
    Pop(V),
    Update(V),
}
