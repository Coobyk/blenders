//! Port of `timeit.cc` — `timeit::ScopedTimer`, `print_duration`.

use std::time::{Duration, Instant};

/// `Nanoseconds` alias.
pub type Nanoseconds = i64;

/// `format_duration` — human readable ns/ms/s string.
pub fn format_duration(duration_ns: i64) -> String {
    let d = Duration::from_nanos(duration_ns.max(0) as u64);
    if duration_ns < 100_000 {
        format!("{} ns", duration_ns)
    } else if d.as_secs() < 5 {
        format!("{:.2} ms", duration_ns as f64 / 1.0e6)
    } else if d.as_secs() > 90 {
        let h = d.as_secs() / 3600;
        let m = (d.as_secs() % 3600) / 60;
        let s = d.as_secs() % 60;
        format!("{:.1} s ({h}H:{m}m:{s}s)", duration_ns as f64 / 1.0e9)
    } else {
        format!("{:.1} s", duration_ns as f64 / 1.0e9)
    }
}

/// `timeit::print_duration`.
pub fn print_duration(duration_ns: i64) {
    print!("{}", format_duration(duration_ns));
}

/// `timeit::ScopedTimer` — prints `Timer '<name>' took <dur>` on drop.
pub struct ScopedTimer {
    name: String,
    start: Instant,
}

impl ScopedTimer {
    pub fn new(name: &str) -> Self {
        ScopedTimer {
            name: name.to_string(),
            start: Instant::now(),
        }
    }
}
impl Drop for ScopedTimer {
    fn drop(&mut self) {
        let dur = self.start.elapsed().as_nanos() as i64;
        println!("Timer '{}' took {}", self.name, format_duration(dur));
    }
}

/// `timeit::ScopedTimerAveraged` — accumulates into caller's counters.
pub struct ScopedTimerAveraged {
    name: String,
    start: Instant,
    window_size: Option<i64>,
}

impl ScopedTimerAveraged {
    pub fn new(name: &str, window_size: Option<i64>) -> Self {
        debug_assert!(window_size.map_or(true, |w| w >= 2));
        ScopedTimerAveraged {
            name: name.to_string(),
            start: Instant::now(),
            window_size,
        }
    }
    /// `~ScopedTimerAveraged` — returns the elapsed ns for the caller to
    /// fold into `total_count`/`total_time`/`min_time`.
    pub fn finish(&self) -> i64 {
        self.start.elapsed().as_nanos() as i64
    }
}
