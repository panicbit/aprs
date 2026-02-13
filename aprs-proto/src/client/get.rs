use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Get {
    pub keys: SmallVec<[String; 1]>,
}
