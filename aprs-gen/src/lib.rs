// base: 53bc3754c0fa50849cba1dc829c9b18a980a3391

use std::sync::LazyLock;

use aprs_utils::collections::Map;

mod world;
use world::{NewPlayerWorld, NewWorld, World};

mod base;
mod options;
mod utils;
mod worlds;

pub type LazyMap<K, V> = LazyLock<Map<K, V>>;
pub type StaticMap<K, V> = &'static LazyMap<K, V>;
