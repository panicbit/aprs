use serde::Serialize;

use crate::{Int, Value, dict, set};

impl Serialize for Value {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::List(list) => ser.collect_seq(list.read().iter()),
            Value::Dict(dict) => ser.collect_map(dict.read().iter()),
            Value::Str(str) => ser.serialize_str(str),
            Value::Int(int) => match *int {
                Int::I64(n) => ser.serialize_i64(n),
                Int::I128(n) => ser.serialize_i128(n),
                Int::BigInt(ref n) => {
                    if let Ok(n) = u128::try_from(n) {
                        ser.serialize_u128(n)
                    } else {
                        ser.serialize_str(&n.to_string())
                    }
                }
            },
            Value::Float(n) => ser.serialize_f64(**n),
            Value::Bool(bool) => ser.serialize_bool(**bool),
            Value::Tuple(tuple) => ser.collect_seq(tuple),
            Value::Callable(callable) => ser.serialize_str(&format!("{callable:?}")),
            Value::None(_) => ser.serialize_none(),
            Value::Set(set) => ser.collect_seq(set.read().iter()),
        }
    }
}

impl Serialize for dict::Key<'_> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            dict::Key::Value(value) => value.serialize(ser),
            dict::Key::Int64(n) => ser.serialize_i64(*n),
        }
    }
}

impl Serialize for set::Item<'_> {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            set::Item::Value(value) => value.serialize(ser),
            set::Item::Int64(n) => ser.serialize_i64(*n),
        }
    }
}
