use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher;
use indexmap::{IndexMap, IndexSet};

#[doc(hidden)]
pub use indexmap::indexmap_with_default as __index_map_with_default;
pub use indexmap::indexset_with_default as __index_set_with_default;

pub type Map<K, V> = FxIndexMap<K, V>;
pub type Set<K> = FxIndexSet<K>;
pub type FxHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxHashSet<K> = HashSet<K, BuildHasherDefault<FxHasher>>;
pub type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
pub type FxIndexSet<K> = IndexSet<K, BuildHasherDefault<FxHasher>>;

#[macro_export]
macro_rules! fx_index_map {
    ($($map:tt)*) => {
        $crate::collections::__index_map_with_default! {
            $crate::hasher::FxHasher;
            $($map)*
        }
    };
}

#[macro_export]
macro_rules! fx_set {
    ($($map:tt)*) => {
        $crate::collections::__index_set_with_default! {
            $crate::hasher::FxHasher;
            $($map)*
        }
    };
}

pub use fx_index_map as map;
pub use fx_set as set;