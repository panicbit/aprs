#![allow(clippy::let_and_return)]

use fnv::FnvBuildHasher;
use indexmap::{IndexMap, IndexSet};

pub mod config;
pub mod game;
pub mod pickle;
pub mod proto;
pub mod server;
pub mod websocket_server;

type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;
type FnvIndexSet<K> = IndexSet<K, FnvBuildHasher>;
