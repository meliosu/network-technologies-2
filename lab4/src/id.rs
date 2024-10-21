use std::sync::atomic::{AtomicI64, Ordering};

pub struct Generator {
    inner: AtomicI64,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            inner: AtomicI64::new(0),
        }
    }

    pub fn next(&self) -> i64 {
        self.inner.fetch_add(1, Ordering::Relaxed)
    }

    pub fn get(&self) -> i64 {
        self.inner.load(Ordering::Relaxed)
    }

    pub fn set(&self, value: i64) {
        self.inner.store(value, Ordering::Relaxed);
    }
}
