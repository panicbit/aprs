use serde::Serialize;

use crate::pickle::Value;

use super::number::N;

impl Serialize for Value {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::List(list) => ser.collect_seq(list),
            Value::Dict(dict) => ser.collect_map(dict),
            Value::Str(str) => ser.serialize_str(str),
            Value::Number(number) => match *number.inner() {
                N::I64(n) => ser.serialize_i64(n),
                N::I128(n) => ser.serialize_i128(n),
                N::BigInt(ref n) => {
                    if let Ok(n) = u128::try_from(n) {
                        ser.serialize_u128(n)
                    } else {
                        ser.serialize_str(&n.to_string())
                    }
                }
                N::F64(n) => ser.serialize_f64(n),
            },
            Value::Bool(bool) => ser.serialize_bool(**bool),
            Value::Tuple(tuple) => ser.collect_seq(tuple),
            Value::Callable(callable) => ser.serialize_str(&format!("{callable:?}")),
            Value::None(_) => ser.serialize_none(),
            Value::Set(set) => ser.collect_seq(set),
        }
    }
}
