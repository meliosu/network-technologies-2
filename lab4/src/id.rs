use std::sync::atomic::{AtomicI32, Ordering};

pub struct Generator {
    curr: AtomicI32,
}

impl Generator {
    pub const fn new() -> Self {
        Self {
            curr: AtomicI32::new(1),
        }
    }

    pub fn next(&self) -> i32 {
        self.curr.fetch_add(1, Ordering::Relaxed)
    }
}
