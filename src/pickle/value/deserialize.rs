use anyhow::{anyhow, bail};
use core::fmt;
use dumpster::sync::Gc;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use serde::{Deserializer, de, forward_to_deserialize_any};
use std::error::Error as StdError;

use crate::pickle::value::{List, list};

use super::{Value, number};

pub type Result<T> = std::result::Result<T, Error>;

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Dict(value) => todo!(),
            Value::List(list) => visitor.visit_seq(ListAccess(list.clone())),
            Value::Str(value) => visitor.visit_str(value.as_str()),
            Value::Number(number) => match number.0 {
                number::Inner::I64(n) => visitor.visit_i64(n),
                number::Inner::I128(n) => visitor.visit_i128(n),
                number::Inner::BigInt(ref big_int) => visitor.visit_string(big_int.to_string()),
                number::Inner::F64(n) => visitor.visit_f64(n),
            },
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::Tuple(value) => todo!(),
            Value::Callable(callable) => visitor.visit_string(format!("{:?}", callable)),
            Value::None(_) => visitor.visit_none(),
            Value::Set(value) => todo!(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

// pub struct ListAccess {
//     list: Gc<List>,
//     index: usize,
//     max_len: usize,
// }

// impl<'de> SeqAccess<'de> for ListAccess {
//     type Error = Error;

//     fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
//     where
//         T: DeserializeSeed<'de>,
//     {
//         let Some(value) = self.next() else {
//             return Ok(None);
//         };

//         let value = seed.deserialize(value)?;

//         Ok(Some(value))
//     }
// }

pub struct Error(anyhow::Error);

impl Error {
    pub fn msg(msg: impl fmt::Display) -> Error {
        Self(anyhow!("{msg}"))
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Self(anyhow!("{msg}"))
    }
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
