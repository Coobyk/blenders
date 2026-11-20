//! Port of `memory_counter.cc` — `blender::memory_counter` shared-data tracking.

/// `MemoryCount` — accumulated totals plus the set of shared blocks already counted.
#[derive(Default)]
pub struct MemoryCount {
    pub total_bytes: i64,
    handled_shared_data: std::collections::HashSet<usize>,
}

/// `MemoryCounter`.
pub struct MemoryCounter<'a> {
    count: &'a mut MemoryCount,
}

impl<'a> MemoryCounter<'a> {
    pub fn new(count: &'a mut MemoryCount) -> Self {
        MemoryCounter { count }
    }
    /// `counter.add(bytes)`.
    pub fn add(&mut self, bytes: i64) {
        self.count.total_bytes += bytes;
    }
    /// `counter.add_shared(sharing_info, bytes)` — counted once per sharing info.
    /// `sharing_info` is identified by its address in C; callers pass a key.
    pub fn add_shared(&mut self, sharing_key: usize, bytes: i64) {
        if !self.count.handled_shared_data.insert(sharing_key) {
            return;
        }
        self.add(bytes);
    }
    /// `add_shared` with a counting callback.
    pub fn add_shared_fn<F: FnOnce(&mut MemoryCounter)>(&mut self, sharing_key: usize, f: F) {
        if !self.count.handled_shared_data.insert(sharing_key) {
            return;
        }
        f(self);
    }
}

impl MemoryCount {
    /// `count.reset()`.
    pub fn reset(&mut self) {
        *self = MemoryCount::default();
    }
}
