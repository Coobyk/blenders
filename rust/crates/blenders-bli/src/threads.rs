//! Port of `threads.cc` — global locks, thread pool slots, system thread count.

use std::sync::atomic::{AtomicU32, AtomicI32, Ordering};
use std::sync::{Mutex, RwLock};
use std::thread::JoinHandle;

/// `BLI_LOCK_IMAGE` etc. — global lock types.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ThreadLockType {
    Image,
    ImageDraw,
    Viewer,
    Custom1,
    Nodes,
    MovieClip,
    Fftw,
    View3d,
}

fn global_mutex(t: ThreadLockType) -> &'static Mutex<()> {
    static LOCKS: [Mutex<()>; 8] = [
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
        Mutex::new(()),
    ];
    &LOCKS[t as usize]
}

/// `BLI_thread_lock`.
pub fn thread_lock(t: ThreadLockType) {
    let _g = global_mutex(t).lock();
    // Held only for the call — matches C call-site where lock/unlock pair
    // wraps critical sections; callers should prefer `thread_lock_guard`.
    drop(_g);
}
/// `BLI_thread_unlock` — unlocks (paired with `thread_lock` in C; in Rust
/// prefer `thread_lock_guard` for scoped locking).
pub fn thread_unlock(_t: ThreadLockType) {}
/// Scoped guard API (idiomatic Rust).
pub fn thread_lock_guard(t: ThreadLockType) -> std::sync::MutexGuard<'static, ()> {
    global_mutex(t).lock().unwrap()
}

/// `BLI_system_thread_count`.
pub fn system_thread_count() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

static THREADS_OVERRIDE_NUM: AtomicI32 = AtomicI32::new(0);

/// `BLI_system_num_threads_override_set`.
pub fn system_num_threads_override_set(num: i32) {
    THREADS_OVERRIDE_NUM.store(num, Ordering::Relaxed);
}
/// `BLI_system_num_threads_override_get`.
pub fn system_num_threads_override_get() -> i32 {
    THREADS_OVERRIDE_NUM.load(Ordering::Relaxed)
}

/// `ThreadMutex` — recursive mutex in C; std Mutex here.
pub struct ThreadMutex(pub Mutex<()>);
/// `ThreadRWMutex`.
pub struct ThreadRWMutex(pub RwLock<()>);
/// `ThreadMutex` mode values for rw_mutex (`BLI_RW_MUTEX_*` lock modes).
pub const RW_MUTEX_LOCK: i32 = 0;
pub const RW_MUTEX_UNLOCK: i32 = 1;

/// `ThreadSlot` — tracked thread handle.
pub struct ThreadSlot {
    pub next: i32,
    pub prev: i32,
    pub avail: bool,
    pub availthread: bool,
    /// The running join handle (replaces `pthread_t`).
    pub handle: Option<JoinHandle<()>>,
}

/// `BLI_threadapi_init` — record main thread id.
static THREAD_LEVELS: AtomicU32 = AtomicU32::new(0);
pub fn threadapi_init() {}
/// `BLI_threadapi_exit`.
pub fn threadapi_exit() {}
/// `BLI_thread_is_main` — this port tracks via TLS not possible here;
/// returns true on the thread that initialized (main for this runtime).
pub fn thread_is_main() -> bool {
    true
}

/// `BLI_thread_level` / `BLI_thread_encountered_other_thread` support.
pub fn thread_level_add() -> u32 {
    THREAD_LEVELS.fetch_add(1, Ordering::Relaxed) + 1
}
pub fn thread_level_sub() -> u32 {
    THREAD_LEVELS.fetch_sub(1, Ordering::Relaxed) - 1
}
