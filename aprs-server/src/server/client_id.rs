use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ClientId(u64);

impl ClientId {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        let id = COUNTER.fetch_add(1, Ordering::SeqCst);

        Self(id)
    }
}
