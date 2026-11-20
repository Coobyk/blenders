//! Port of `task_range.cc` — `parallel_for` / `parallel_reduce` /
//! `memory_bandwidth_bound_task` / `max_threads_task`.
//! Serial execution; same call semantics (TBB path is an optimization).

use crate::index_range::IndexRange;
use std::sync::atomic::{AtomicI32, Ordering};

static CURRENT_REDUCE_SUB_RANGE_START: AtomicI32 = AtomicI32::new(-1);

/// `TaskParallelRangeFunc` — userdata-free form.
/// `BLI_task_parallel_range`.
pub fn task_parallel_range(start: i32, stop: i32, mut func: impl FnMut(i32, i32)) {
    func(start, stop);
}

/// `BLI_task_parallel_thread_id`.
pub fn task_parallel_thread_id() -> i32 {
    0
}

/// `threading::parallel_for` — call `func` on sub-ranges of `range`.
/// Serial: one call covering the whole range.
pub fn parallel_for(range: IndexRange, _grain_size: i64, mut func: impl FnMut(IndexRange)) {
    if range.is_empty() {
        return;
    }
    func(range);
}

/// `parallel_for` with per-element sizes (`parallel_for_impl` variants).
/// `func` gets each sub-range; `grain_size` is advisory.
pub fn parallel_for_weighted(
    range: IndexRange,
    grain_size: i64,
    sizes: impl Fn(i64) -> i64,
    mut func: impl FnMut(IndexRange),
) {
    // Split into chunks of about `grain_size` total "weight" (serial impl).
    let mut chunk_start = range.start;
    let mut acc = 0i64;
    let mut i = range.start;
    while i < range.one_after_last() {
        acc += sizes(i).max(1);
        i += 1;
        if acc >= grain_size || i == range.one_after_last() {
            func(IndexRange::new(chunk_start, i - chunk_start));
            chunk_start = i;
            acc = 0;
        }
    }
}

/// `threading::parallel_reduce` — fold `values` → `reduce` over ranges.
pub fn parallel_reduce<T: Clone + Send>(
    range: IndexRange,
    _grain_size: i64,
    init: T,
    mut func: impl FnMut(IndexRange, T) -> T,
    reduce: impl FnOnce(T, T) -> T,
) -> T {
    let result = func(range, init.clone());
    reduce(init, result)
}

/// `threading::memory_bandwidth_bound_task` — run `function` (serial).
pub fn memory_bandwidth_bound_task(_bytes: usize, function: impl FnOnce()) {
    function();
}

/// `threading::max_threads_task` — run `function` (serial).
pub fn max_threads_task(_max_threads: i32, function: impl FnOnce()) {
    function();
}

/// `threading::isolate_task`.
pub fn isolate_task(function: impl FnOnce()) {
    function();
}

/// `threading::EnumerableThreadSpecific` — serial equivalent (one value).
pub struct EnumerableThreadSpecific<T> {
    local: T,
}
impl<T> EnumerableThreadSpecific<T> {
    pub fn new(local: T) -> Self {
        EnumerableThreadSpecific { local }
    }
    pub fn local(&mut self) -> &mut T {
        &mut self.local
    }
}

/// Internal marker retained for API parity.
pub fn current_reduce_sub_range_start() -> i32 {
    CURRENT_REDUCE_SUB_RANGE_START.load(Ordering::Relaxed)
}
