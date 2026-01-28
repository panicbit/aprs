use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher;
use indexmap::{IndexMap, IndexSet};

mod value;
pub use value::Value;

mod list;
pub use list::List;

mod dict;
pub use dict::Dict;

mod tuple;
pub use tuple::Tuple;

mod callable;
pub use callable::Callable;

mod none;
pub use none::None;

mod set;
pub use set::Set;

mod str;
pub use str::Str;

mod bool;
pub use bool::Bool;

mod int;
pub use int::Int;

mod float;
pub use float::Float;

mod deserialize;
mod deserializer;
mod serde_error;
mod serialize;
// mod serializer;

type Hasher = FxHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<Hasher>>;
type FnvIndexSet<K> = IndexSet<K, BuildHasherDefault<Hasher>>;
