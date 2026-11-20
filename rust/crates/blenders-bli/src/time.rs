//! Port of `time.cc` — monotonic clock + sleep helpers.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// Epoch stand-in for `steady_clock`'s arbitrary time base: first call.
fn epoch() -> Instant {
    static T0: OnceLock<Instant> = OnceLock::new();
    *T0.get_or_init(Instant::now)
}

/// `BLI_time_now_seconds` — steady clock, seconds as f64.
pub fn time_now_seconds() -> f64 {
    Instant::now().duration_since(epoch()).as_secs_f64()
}

/// `BLI_time_now_seconds_i` — steady clock, whole seconds.
pub fn time_now_seconds_i() -> i64 {
    time_now_seconds() as i64
}

/// `BLI_time_sleep_ms` (POSIX branch).
pub fn time_sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// `BLI_time_sleep_precise_us` (POSIX branch).
pub fn time_sleep_precise_us(us: u64) {
    std::thread::sleep(Duration::from_micros(us));
}
