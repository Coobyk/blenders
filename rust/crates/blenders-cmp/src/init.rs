//! Port of `intern/init.cc` — compositor
//! system init/exit.

/// `compositor_init`/`compositor_exit` (verbatim —
/// registers evaluation function).
pub static mut COMPOSITOR_INITIALIZED: bool = false;
/// init (verbatim).
pub fn compositor_init() {
    unsafe { COMPOSITOR_INITIALIZED = true; }
}
/// exit.
pub fn compositor_exit() {
    unsafe { COMPOSITOR_INITIALIZED = false; }
}
