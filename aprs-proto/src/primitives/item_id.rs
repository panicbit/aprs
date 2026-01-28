use serde::{Deserialize, Serialize};

use crate::deserialize::i64_or_string;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ItemId(#[serde(deserialize_with = "i64_or_string")] pub i64);
