//! Port of `intern/date_string.cc` — localized
//! date/time format strings.

/// `DateFormat`/`TimeFormat` verbatim ordinals.
#[derive(Clone, Copy)]
pub enum DateFormat {
    /// short.
    Short,
    /// long (with weekday).
    Long,
}
#[derive(Clone, Copy)]
pub enum TimeFormat {
    /// hh:mm.
    WithoutSeconds,
    /// hh:mm:ss.
    WithSeconds,
}
/// `date`/`time`/`datetime` format string assembly
/// (verbatim — per-locale pattern order + separator).
pub fn format_datetime(
    pattern_date_first: bool,
    date_sep: &str,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
    time_format: TimeFormat,
) -> String {
    let date = format!("{:04}{}{:02}{}{:02}", year, date_sep, month, date_sep, day);
    let time = match time_format {
        TimeFormat::WithoutSeconds => format!("{:02}:{:02}", hour, min),
        TimeFormat::WithSeconds => format!("{:02}:{:02}:{:02}", hour, min, sec),
    };
    if pattern_date_first {
        format!("{} {}", date, time)
    } else {
        format!("{} {}", time, date)
    }
}
