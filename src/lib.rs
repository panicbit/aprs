#![allow(clippy::let_and_return)]

use std::hash::BuildHasherDefault;

use hashers::fx_hash::FxHasher;
use indexmap::{IndexMap, IndexSet};

pub mod game;
pub mod pickle;
pub mod proto;
pub mod server;
pub mod websocket_server;

type Hasher = FxHasher;
type FnvIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<Hasher>>;
type FnvIndexSet<K> = IndexSet<K, BuildHasherDefault<Hasher>>;
