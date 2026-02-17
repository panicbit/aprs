use std::ffi::CString;

use itertools::Itertools;
use num::BigInt;
use ordered_float::OrderedFloat;
use pyo3::types::{PyAnyMethods, PyDict, PyList, PySet, PyTuple, PyTypeMethods};
use pyo3::{Bound, PyAny, Python};
use serde::{Deserialize, Serialize};
use strum::VariantArray;

use crate::int::Minimize;
use crate::{Dict, List, Set, Tuple, Value};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
enum PythonValue {
    Int(BigInt),
    Float(OrderedFloat<f64>),
    Bool(bool),
    Str(String),
    List(Vec<PythonValue>),
    Dict(Vec<(PythonValue, PythonValue)>),
    Set(Vec<PythonValue>),
    Tuple(Vec<PythonValue>),
    None,
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
            Self::None => "None".to_string(),
        }
    }

    fn to_rust(&self) -> Value {
        match self {
            Self::Int(value) => Value::Int(value.minimize()),
            Self::Float(value) => Value::float(value.0),
            Self::Bool(value) => Value::bool(*value),
            Self::Str(value) => Value::str(value),
            Self::List(value) => {
                let value = value.iter().map(|v| v.to_rust()).collect::<List>();

                Value::List(value)
            }
            Self::Dict(value) => {
                let value = value.iter().map(|(k, v)| (k.to_rust(), v.to_rust()));
                let value = Dict::try_from_iter(value).expect("cannot construct dict");

                Value::Dict(value)
            }
            Self::Set(value) => {
                let value = value.iter().map(|v| v.to_rust());
                let value = Set::try_from_iter(value).expect("cannot construct set");

                Value::Set(value)
            }
            Self::Tuple(value) => {
                let value = value.iter().map(|v| v.to_rust()).collect::<Tuple>();

                Value::Tuple(value)
            }
            Self::None => Value::none(),
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

impl From<Value> for PythonValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Dict(value) => PythonValue::Dict(
                value
                    .read()
                    .iter()
                    .map(|(k, v)| {
                        let k = PythonValue::from(Value::from(k));
                        let v = PythonValue::from(Value::from(v));

                        (k, v)
                    })
                    .collect(),
            ),
            Value::List(value) => PythonValue::List(
                value
                    .read()
                    .iter()
                    .cloned()
                    .map(PythonValue::from)
                    .collect(),
            ),
            Value::Str(value) => PythonValue::Str(value.as_str().to_owned()),
            Value::Int(value) => PythonValue::Int(value.to_big_int()),
            Value::Float(value) => PythonValue::Float(OrderedFloat(*value)),
            Value::Bool(value) => PythonValue::Bool(*value),
            Value::Tuple(value) => {
                PythonValue::Tuple(value.iter().cloned().map(PythonValue::from).collect())
            }
            Value::Callable(_) => panic!("cannot convert Callable to PythonValue"),
            Value::None(_) => PythonValue::None,
            Value::Set(value) => PythonValue::Set(
                value
                    .read()
                    .iter()
                    .map(Value::from)
                    .map(PythonValue::from)
                    .collect(),
            ),
        }
    }
}

macro_rules! list {
    [$($expr:expr),*$(,)?] => {
        PythonValue::List(vec![$($expr.into()),*])
    };
}

fn eval_python(code: &str) -> Result<PythonValue, String> {
    let code = CString::new(code).unwrap();

    Python::attach(|py| {
        py.eval(&code, None, None)
            .map(pyo3_value_to_python_value)
            .map_err(|err| err.to_string())
    })
}

fn pyo3_value_to_python_value(value: Bound<PyAny>) -> PythonValue {
    let qualname = value
        .get_type()
        .fully_qualified_name()
        .expect("failed to get qualified name");
    let qualname = qualname
        .extract::<&str>()
        .expect("expected qualname to be a &str");

    match qualname {
        "int" => {
            let value = value
                .extract::<BigInt>()
                .expect("expected value to be an int");

            PythonValue::Int(value)
        }
        "float" => {
            let value = value
                .extract::<f64>()
                .expect("expected value to be a float");

            PythonValue::Float(OrderedFloat(value))
        }
        "bool" => {
            let value = value
                .extract::<bool>()
                .expect("expected value to be a bool");

            PythonValue::Bool(value)
        }
        "str" => {
            let value = value
                .extract::<String>()
                .expect("expected value to be a string");

            PythonValue::Str(value)
        }
        "list" => {
            let value = value.cast::<PyList>().expect("expected value to be a list");
            let value = value
                .into_iter()
                .map(|value| pyo3_value_to_python_value(value.clone()))
                .collect::<Vec<_>>();

            PythonValue::List(value)
        }
        "dict" => {
            let value = value.cast::<PyDict>().expect("expected value to be a dict");
            let value = value
                .into_iter()
                .map(|(k, v)| {
                    let k = pyo3_value_to_python_value(k.clone());
                    let v = pyo3_value_to_python_value(v.clone());

                    (k, v)
                })
                .collect::<Vec<_>>();

            PythonValue::Dict(value)
        }
        "set" => {
            let value = value.cast::<PySet>().expect("expected value to be a set");
            let value = value
                .into_iter()
                .map(|value| pyo3_value_to_python_value(value.clone()))
                .collect::<Vec<_>>();

            PythonValue::Set(value)
        }
        "tuple" => {
            let value = value
                .cast::<PyTuple>()
                .expect("expected value to be a tuple");
            let value = value
                .into_iter()
                .map(|value| pyo3_value_to_python_value(value.clone()))
                .collect::<Vec<_>>();

            PythonValue::Tuple(value)
        }
        "NoneType" => PythonValue::None,
        _ => panic!("unhandled type: {qualname}"),
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

    fn eval_rust(&self, lhs: &PythonValue, rhs: &PythonValue) -> Result<PythonValue, String> {
        let lhs = lhs.to_rust();
        let rhs = rhs.to_rust();

        match self {
            BinaryOp::Add => lhs
                .add(&rhs)
                .map(PythonValue::from)
                .map_err(|err| err.to_string()),
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
                let python_result = eval_python(&code);
                let rust_result = binop.eval_rust(lhs, rhs);

                eprintln!("Code: {code}");

                match (rust_result, python_result) {
                    (Ok(rust), Ok(python)) => {
                        assert_eq!(rust, python)
                    }
                    (Ok(rust), Err(python)) => {
                        panic!("  Rust: {rust:?}\nPython: {python:?}");
                    }
                    (Err(rust), Ok(python)) => {
                        panic!("  Rust: {rust:?}\nPython: {python:?}");
                    }
                    (Err(rust), Err(python)) => {
                        // TODO: figure out how to verify these are the same
                        eprintln!("  Rust: {rust:?}\nPython: {python:?}");
                    }
                }
            }
        }
    }
}
