//! Port of `task_iterator.cc` — `BLI_task_parallel_mempool` iterator.

use std::sync::Mutex;

/// `TaskParallelMempoolIterateFunc` — called per element.
/// `BLI_task_parallel_mempool` — iterate `mempool` elements calling `func`.
/// (`use_userdata`/`use_threads` preserved as API surface; runs serially.)
pub fn task_parallel_mempool<T, F: Fn(&T)>(
    mempool: &crate::mempool::Mempool<T>,
    func: F,
    _use_userdata: bool,
    _use_threads: bool,
) {
    let mut it = mempool.iter();
    while let Some(elem) = it.next_elem() {
        func(elem);
    }
}

/// `BLI_task_parallel_mempool` with userdata slot + shared userdata
/// (`ParallelMempoolState`).
pub fn task_parallel_mempool_with_settings<T, U, F: Fn(&T, &U) + Sync>(
    mempool: &crate::mempool::Mempool<T>,
    userdata: &U,
    func: F,
    _settings: &TaskParallelSettings,
) {
    let f = Mutex::new(&func);
    let mut it = mempool.iter();
    while let Some(elem) = it.next_elem() {
        (f.lock().unwrap())(elem, userdata);
    }
}

/// `TaskParallelSettings` (task_iterator.h).
#[derive(Clone, Copy)]
pub struct TaskParallelSettings {
    pub func_finalize: bool,
    pub use_threading: bool,
    pub userdata_chunk: usize,
    pub userdata_chunk_size: usize,
    pub min_iter_per_thread: usize,
}
