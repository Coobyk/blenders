//! Port of `task_scheduler.cc` — global scheduler init/num-threads.
//! Rust runs parallel work via std::thread scoped threads; the scheduler
//! state tracks the configured thread count like TBB's global_control.

use std::sync::atomic::{AtomicI32, Ordering};

static TASK_SCHEDULER_NUM_THREADS: AtomicI32 = AtomicI32::new(1);

/// `BLI_task_scheduler_init`.
pub fn task_scheduler_init(_use_flush_denormals_to_zero: bool) {
    // Set to hardware concurrency (C uses tbb default arena).
    let n = std::thread::available_parallelism()
        .map(|n| n.get() as i32)
        .unwrap_or(1);
    TASK_SCHEDULER_NUM_THREADS.store(n, Ordering::Relaxed);
}

/// `BLI_task_scheduler_exit`.
pub fn task_scheduler_exit() {
    TASK_SCHEDULER_NUM_THREADS.store(1, Ordering::Relaxed);
}

/// `BLI_task_scheduler_num_threads`.
pub fn task_scheduler_num_threads() -> i32 {
    let n = TASK_SCHEDULER_NUM_THREADS.load(Ordering::Relaxed);
    if n == 0 {
        1
    } else {
        n
    }
}

/// `BLI_task_isolate` — run `func` in isolation (serial here).
pub fn task_isolate(func: impl FnOnce()) {
    func();
}
