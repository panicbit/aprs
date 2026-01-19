use serde::de::value::{MapDeserializer, SeqDeserializer, StrDeserializer};
use serde::de::{IntoDeserializer, Visitor};
use serde::{Deserializer, forward_to_deserialize_any};

use crate::pickle::value::serde_error::SerdeError;
use crate::pickle::value::{Int, Storage, dict, set};

use super::Value;

pub type Result<T> = std::result::Result<T, SerdeError>;

impl<'de, S: Storage> Deserializer<'de> for &Value<S> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Dict(dict) => visitor.visit_map(MapDeserializer::new(dict.read().iter())),
            Value::List(list) => visitor.visit_seq(SeqDeserializer::new(list.read().iter())),
            Value::Str(str) => visitor.visit_str(str.as_str()),
            Value::Int(int) => match *int {
                Int::I64(n) => visitor.visit_i64(n),
                Int::I128(n) => visitor.visit_i128(n),
                Int::BigInt(ref n) => visitor.visit_string(n.to_string()),
            },
            Value::Float(n) => visitor.visit_f64(**n),
            Value::Bool(bool) => visitor.visit_bool(**bool),
            Value::Tuple(tuple) => visitor.visit_seq(SeqDeserializer::new(tuple.iter())),
            Value::Callable(callable) => visitor.visit_string(format!("{:?}", callable)),
            Value::None(_) => visitor.visit_none(),
            Value::Set(set) => visitor.visit_seq(SeqDeserializer::new(set.read().iter())),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Dict(_) => self.deserialize_any(visitor),
            Value::List(_) => self.deserialize_any(visitor),
            Value::Str(str) => StrDeserializer::new(str).deserialize_enum(name, variants, visitor),
            Value::Int(_) => self.deserialize_any(visitor),
            Value::Float(_) => self.deserialize_any(visitor),
            Value::Bool(_) => self.deserialize_any(visitor),
            Value::Tuple(_) => self.deserialize_any(visitor),
            Value::Callable(_) => self.deserialize_any(visitor),
            Value::None(_) => self.deserialize_any(visitor),
            Value::Set(_) => self.deserialize_any(visitor),
        }
    }

    // fn deserialize_tuple_struct<V>(
    //     self,
    //     name: &'static str,
    //     len: usize,
    //     visitor: V,
    // ) -> Result<V::Value>
    // where
    //     V: Visitor<'de>,
    // {
    //     match self {
    //         Value::Dict(_) => self.deserialize_any(visitor),
    //         Value::List(_) => self.deserialize_any(visitor),
    //         Value::Str(str) => {
    //             panic!("here: {name}");
    //             StrDeserializer::new(&str).deserialize_tuple_struct(name, len, visitor)
    //         }
    //         Value::Number(_) => {
    //             panic!("here: {name}");
    //             self.deserialize_any(visitor)
    //         }
    //         Value::Bool(_) => self.deserialize_any(visitor),
    //         Value::Tuple(_) => self.deserialize_any(visitor),
    //         Value::Callable(_) => self.deserialize_any(visitor),
    //         Value::None(_) => self.deserialize_any(visitor),
    //         Value::Set(_) => self.deserialize_any(visitor),
    //     }
    // }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::None(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq struct
        map identifier ignored_any tuple
    }
}

impl<S: Storage> IntoDeserializer<'_, SerdeError> for &Value<S> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de, S: Storage> Deserializer<'de> for dict::Key<'_, S> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            dict::Key::Value(value) => value.deserialize_any(visitor),
            dict::Key::Int64(n) => visitor.visit_i64(n),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        enum option
        bool i8 i16 i32 i64 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq struct
        map identifier ignored_any tuple
    }
}

impl<S: Storage> IntoDeserializer<'_, SerdeError> for dict::Key<'_, S> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de, S: Storage> Deserializer<'de> for set::Item<'_, S> {
    type Error = SerdeError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            set::Item::Value(value) => value.deserialize_any(visitor),
            set::Item::Int64(n) => visitor.visit_i64(n),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    forward_to_deserialize_any! {
        enum option
        bool i8 i16 i32 i64 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq struct
        map identifier ignored_any tuple
    }
}

impl<S: Storage> IntoDeserializer<'_, SerdeError> for set::Item<'_, S> {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
