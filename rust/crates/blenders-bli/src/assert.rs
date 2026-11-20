//! Port of `BLI_assert.cc` — assertion print/abort helpers.
//! Rust uses `debug_assert!`/`panic!`; these functions provide C-parity hooks.

use std::sync::atomic::{AtomicBool, Ordering};

static WITH_ASSERT_ABORT: AtomicBool = AtomicBool::new(true);
static WITH_ASSERT_BACKTRACE: AtomicBool = AtomicBool::new(true);

/// `BLI_assert_set_abort`.
pub fn assert_set_abort(v: bool) {
    WITH_ASSERT_ABORT.store(v, Ordering::Relaxed);
}
/// `BLI_assert_set_backtrace`.
pub fn assert_set_backtrace(v: bool) {
    WITH_ASSERT_BACKTRACE.store(v, Ordering::Relaxed);
}
/// `_BLI_assert_print_pos`.
pub fn assert_print_pos(file: &str, line: u32, function: &str, id: &str) {
    eprintln!("BLI_assert: {file}:{line}: {function}(): '{id}'");
}
/// `_BLI_assert_print_extra`.
pub fn assert_print_extra(s: &str) {
    eprintln!("BLI_assert extra info: {s}");
}
/// `_BLI_assert_unreachable_print`.
pub fn assert_unreachable_print(file: &str, line: u32, function: &str) {
    eprintln!("BLI_assert unreachable: {file}:{line}: {function}()");
}
/// `_BLI_assert_print_backtrace` — `std::backtrace`.
pub fn assert_print_backtrace() {
    if WITH_ASSERT_BACKTRACE.load(Ordering::Relaxed) {
        eprintln!("{}", std::backtrace::Backtrace::capture());
    }
}
/// `_BLI_assert_abort`.
pub fn assert_abort() -> ! {
    if WITH_ASSERT_ABORT.load(Ordering::Relaxed) {
        std::process::abort();
    } else {
        panic!("BLI_assert failed");
    }
}
