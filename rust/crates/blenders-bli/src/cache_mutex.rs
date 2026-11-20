//! Port of `cache_mutex.cc` — `CacheMutex` (double-checked lazy init).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

/// `CacheMutex`.
pub struct CacheMutex {
    cache_valid: AtomicBool,
    mutex: Mutex<()>,
}

impl CacheMutex {
    pub fn new() -> Self {
        CacheMutex {
            cache_valid: AtomicBool::new(false),
            mutex: Mutex::new(()),
        }
    }
    /// `CacheMutex::ensure` — run `compute_cache` once under mutex.
    pub fn ensure(&self, compute_cache: impl FnOnce()) {
        if self.cache_valid.load(Ordering::Acquire) {
            return;
        }
        let _lock = self.mutex.lock().unwrap();
        if self.cache_valid.load(Ordering::Relaxed) {
            return;
        }
        compute_cache();
        self.cache_valid.store(true, Ordering::Release);
    }
    /// `CacheMutex::invalidate`.
    pub fn invalidate(&self) {
        let _lock = self.mutex.lock().unwrap();
        self.cache_valid.store(false, Ordering::Release);
    }
}
impl Default for CacheMutex {
    fn default() -> Self {
        Self::new()
    }
}
