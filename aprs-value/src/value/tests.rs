use std::process::{Command, Stdio};

use itertools::Itertools;
use num::BigInt;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use strum::VariantArray;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
enum PythonValue {
    #[serde(with = "big_int")]
    Int(BigInt),
    #[serde(with = "float")]
    Float(OrderedFloat<f64>),
    Bool(bool),
    Str(String),
    List(Vec<PythonValue>),
    Dict(Vec<(PythonValue, PythonValue)>),
    Set(Vec<PythonValue>),
    Tuple(Vec<PythonValue>),
}

impl PythonValue {
    fn to_python(&self) -> String {
        match self {
            Self::Int(value) => format!("int({value})"),
            Self::Float(value) => {
                let value = **value;

                if value.is_nan() {
                    "float('nan')".to_string()
                } else if value.is_infinite() && value.is_sign_positive() {
                    "float('inf')".to_string()
                } else if value.is_infinite() && value.is_sign_negative() {
                    "float('-inf')".to_string()
                } else {
                    format!("float({value})")
                }
            }
            Self::Bool(value) => format!("bool({})", u8::from(*value)),
            Self::Str(value) => format!("str(\"{value}\")"),
            Self::List(value) => slice_to_python(value),
            Self::Dict(value) => {
                let items = value
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k.to_python(), v.to_python()))
                    .join(", ");
                format!("{{{items}}}")
            }
            Self::Set(value) => format!("set({})", slice_to_python(value)),
            Self::Tuple(value) => format!("tuple({})", slice_to_python(value)),
        }
    }

    fn as_list(&self) -> &[PythonValue] {
        match self {
            PythonValue::List(list) => list,
            _ => panic!("not a list: {self:?}"),
        }
    }
}

fn slice_to_python(slice: &[PythonValue]) -> String {
    let items = slice.iter().map(|v| v.to_python()).join(", ");
    format!("[{items}]")
}

mod big_int {
    use num::BigInt;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

    pub fn deserialize<'de, D>(de: D) -> Result<BigInt, D::Error>
    where
        D: Deserializer<'de>,
    {
        let int = serde_json::Number::deserialize(de)?;
        let int = int.as_str().parse::<BigInt>().map_err(de::Error::custom)?;

        Ok(int)
    }

    pub fn serialize<S>(value: &BigInt, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.to_string().serialize(ser)
    }
}

mod float {
    use core::f64;

    use ordered_float::OrderedFloat;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
    use serde_json::Number;

    pub fn deserialize<'de, D>(de: D) -> Result<OrderedFloat<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum FloatOrString {
            Number(Number),
            String(String),
        }

        let value = FloatOrString::deserialize(de)?;
        let value = match value {
            FloatOrString::Number(value) => value.as_f64().unwrap(),
            FloatOrString::String(value) => match value.as_str() {
                "NaN" => f64::NAN,
                "inf" => f64::INFINITY,
                "-inf" => f64::NEG_INFINITY,
                _ => return Err(de::Error::custom(format!("invalid float: `{value}`"))),
            },
        };

        Ok(OrderedFloat(value))
    }

    pub fn serialize<S>(value: &OrderedFloat<f64>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = **value;

        if value.is_nan() {
            "NaN".serialize(ser)
        } else if value.is_infinite() && value.is_sign_positive() {
            "inf".serialize(ser)
        } else if value.is_infinite() && value.is_sign_negative() {
            "-inf".serialize(ser)
        } else {
            value.serialize(ser)
        }
    }
}

impl From<u8> for PythonValue {
    fn from(value: u8) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<i8> for PythonValue {
    fn from(value: i8) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<u16> for PythonValue {
    fn from(value: u16) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<i16> for PythonValue {
    fn from(value: i16) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<u32> for PythonValue {
    fn from(value: u32) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<i32> for PythonValue {
    fn from(value: i32) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<u64> for PythonValue {
    fn from(value: u64) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<i64> for PythonValue {
    fn from(value: i64) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<u128> for PythonValue {
    fn from(value: u128) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<i128> for PythonValue {
    fn from(value: i128) -> Self {
        PythonValue::Int(BigInt::from(value))
    }
}

impl From<BigInt> for PythonValue {
    fn from(value: BigInt) -> Self {
        PythonValue::Int(value)
    }
}

impl From<f64> for PythonValue {
    fn from(value: f64) -> Self {
        PythonValue::Float(OrderedFloat(value))
    }
}

impl From<&str> for PythonValue {
    fn from(value: &str) -> Self {
        PythonValue::Str(String::from(value))
    }
}

macro_rules! list {
    [$($expr:expr),*$(,)?] => {
        PythonValue::List(vec![$($expr.into()),*])
    };
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PythonResult {
    Ok(PythonValue),
    Err(String),
}

fn eval(code: &str) -> Result<PythonValue, String> {
    let output = Command::new("python")
        .arg("test_eval.py")
        .arg(code)
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    let result = serde_json::from_slice::<PythonResult>(&output.stdout).unwrap();

    match result {
        PythonResult::Ok(value) => Ok(value),
        PythonResult::Err(err) => Err(err),
    }
}

enum UnaryOp {
    Floor,
    Ceil,
}

#[derive(VariantArray)]
enum BinaryOp {
    Add,
    // Mul,
    // Pow,
    // Mod,
    // Min,
    // Max,
    // And,
    // Or,
    // Pop,
    // Update,
}

impl BinaryOp {
    fn to_python(&self, lhs: &PythonValue, rhs: &PythonValue) -> String {
        let lhs = lhs.to_python();
        let rhs = rhs.to_python();

        match self {
            BinaryOp::Add => format!("{lhs} + {rhs}"),
        }
    }
}

#[test]
fn foo() {
    let values = list![
        BigInt::from(i128::MIN) + BigInt::from(i128::MIN),
        i128::MIN,
        i64::MIN,
        i32::MIN,
        i16::MIN,
        i8::MIN,
        0,
        i8::MAX,
        u8::MAX,
        i16::MAX,
        u16::MAX,
        i32::MAX,
        u32::MAX,
        i64::MAX,
        u64::MAX,
        i128::MAX,
        u128::MAX,
        BigInt::from(u128::MAX) + BigInt::from(u128::MAX),
        "",
        "hi",
        "0",
        "1",
        "42",
        f64::NEG_INFINITY,
        f64::MIN,
        -32,
        -1.0f64,
        0.0f64,
        1.0f64,
        32,
        f64::MAX,
        f64::INFINITY,
        f64::NAN,
    ];
    let values = values.as_list();

    for lhs in values {
        for rhs in values {
            for binop in BinaryOp::VARIANTS {
                let code = binop.to_python(lhs, rhs);
                let result = eval(&code);
                println!("{code} ⇒ {result:?}");
            }
        }
    }

    panic!("TODO");
}
