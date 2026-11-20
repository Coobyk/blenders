//! Port of `math_time.cc` — `BLI_math_time_seconds_decompose`.

const SECONDS_IN_DAY: f64 = 86400.0;
const SECONDS_IN_HOUR: f64 = 3600.0;
const SECONDS_IN_MINUTE: f64 = 60.0;

/// `BLI_math_time_seconds_decompose` — split `seconds` into whichever
/// requested components are non-None, updating all in place.
pub fn math_time_seconds_decompose(
    seconds: f64,
    r_days: Option<&mut f64>,
    r_hours: Option<&mut f64>,
    r_minutes: Option<&mut f64>,
    r_seconds: Option<&mut f64>,
    r_milliseconds: Option<&mut f64>,
) {
    let mut seconds = seconds;
    if let Some(d) = r_days {
        *d = (seconds / SECONDS_IN_DAY).trunc();
        seconds -= *d * SECONDS_IN_DAY;
    }
    if let Some(h) = r_hours {
        *h = (seconds / SECONDS_IN_HOUR).trunc();
        seconds -= *h * SECONDS_IN_HOUR;
    }
    if let Some(m) = r_minutes {
        *m = (seconds / SECONDS_IN_MINUTE).trunc();
        seconds -= *m * SECONDS_IN_MINUTE;
    }
    if let Some(s) = r_seconds {
        *s = seconds.trunc();
        seconds -= *s;
    }
    if let Some(ms) = r_milliseconds {
        *ms = seconds * 1000.0;
    }
}
