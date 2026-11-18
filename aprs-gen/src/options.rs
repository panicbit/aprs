use std::ops::Bound;

pub trait Option {
    const NAME: &str;
    const DESCRIPTION: &str;

    fn deserialize<D: Deserializer>(deserializer: D) -> Self;
}

pub trait Deserializer {
    type Map: MapAccess;

    fn deserialize_toggle(self, default: bool) -> bool;
    fn deserialize_range<N: Number>(self, start: Bound<N>, end: Bound<N>, default: N) -> N;
    fn deserialize_map<F, R, DF>(self, f: F, default: DF) -> R
    where
        F: FnOnce(Self::Map) -> R,
        DF: FnOnce() -> R;
}

pub trait MapAccess {
    type ValueDeserializer: Deserializer;

    fn get(&self, key: &str) -> Self::ValueDeserializer;
}

#[derive(Default)]
pub struct DeathLink {
    enabled: bool,
}

impl Option for DeathLink {
    const NAME: &str = "Death Link";
    const DESCRIPTION: &str =
        "When you die, everyone who enabled death link dies. Of course, the reverse is true too.";

    fn deserialize<D: Deserializer>(deserializer: D) -> Self {
        Self {
            enabled: deserializer.deserialize_toggle(false),
        }
    }
}

pub trait Number {}

impl Number for u32 {}

impl Number for i32 {}

impl Number for i64 {}

impl Number for f64 {}
