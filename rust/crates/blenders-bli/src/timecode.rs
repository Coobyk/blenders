//! Port of `timecode.cc` — timecode string formatting.

/// `eTimecodeStyles` (DNA_userdef_types.h).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TimecodeStyle {
    /// `USER_TIMECODE_MINIMAL` — hh:mm:ss (+frames when brevity>0).
    Minimal,
    /// `USER_TIMECODE_SMPTE_MSF` — mm:ss:ff or hh:mm:ss:ff.
    SmpteMsf,
    /// `USER_TIMECODE_MILLISECONDS` — mm:ss.milliseconds.
    Milliseconds,
    /// `USER_TIMECODE_SECONDS_ONLY`.
    SecondsOnly,
    /// `USER_TIMECODE_SMPTE_FULL` — full HH:MM:SS:FF.
    SmpteFull,
    /// `USER_TIMECODE_SUBRIP` — `HH:MM:SS,mmm`.
    Subrip,
}

/// `BLI_timecode_string_from_time` — format a timecode to a String.
pub fn timecode_string_from_time(
    brevity_level: i32,
    time_seconds: f32,
    fps: f64,
    timecode_style: TimecodeStyle,
) -> String {
    let mut hours = 0i32;
    let mut minutes = 0i32;
    let mut seconds;
    let mut frames = 0i32;
    let mut time = time_seconds;
    let neg = if time < 0.0 {
        time = -time;
        "-"
    } else {
        ""
    };

    if time >= 3600.0 {
        hours = time as i32 / 3600;
        time %= 3600.0;
    }
    if time >= 60.0 {
        minutes = time as i32 / 60;
        time %= 60.0;
    }

    if brevity_level <= 0 {
        seconds = time as i32;
        frames = (((time - seconds as f32) as f64) * fps).round() as i32;
    } else {
        seconds = time.round() as i32;
    }

    match timecode_style {
        TimecodeStyle::Minimal => {
            if brevity_level <= 0 {
                if hours != 0 {
                    format!("{neg}{hours:02}:{minutes:02}:{seconds:02}+{frames:02}")
                } else if minutes != 0 {
                    format!("{neg}{minutes:02}:{seconds:02}+{frames:02}")
                } else {
                    format!("{neg}00:{seconds:02}+{frames:02}")
                }
            } else {
                if hours != 0 {
                    format!("{neg}{hours:02}:{minutes:02}:{seconds:02}")
                } else {
                    format!("{neg}{minutes:02}:{seconds:02}")
                }
            }
        }
        TimecodeStyle::SmpteMsf => {
            if hours != 0 {
                format!("{neg}{hours:02}:{minutes:02}:{seconds:02}:{frames:02}")
            } else {
                format!("{neg}{minutes:02}:{seconds:02}:{frames:02}")
            }
        }
        TimecodeStyle::Milliseconds => {
            let ms_dp = if brevity_level <= 0 { 1 - brevity_level } else { 1 } as usize;
            let s_pad = ms_dp + 3;
            if hours != 0 {
                format!("{neg}{hours:02}:{minutes:02}:{:0width$.prec$}", time, width = s_pad, prec = ms_dp)
            } else {
                format!("{neg}{minutes:02}:{:0width$.prec$}", time, width = s_pad, prec = ms_dp)
            }
        }
        TimecodeStyle::Subrip => {
            let ms_dp = if brevity_level <= 0 { 1 - brevity_level } else { 1 } as usize;
            let ms = ((time - seconds as f32) * 1000.0).round() as i32;
            format!("{neg}{hours:02}:{minutes:02}:{seconds:02},{ms:0ms_dp$}")
        }
        TimecodeStyle::SecondsOnly => {
            if brevity_level <= 0 {
                let prec = (1 - brevity_level) as usize;
                format!("{:.prec$}", time_seconds, prec = prec)
            } else {
                format!("{}", time_seconds.round() as i32)
            }
        }
        TimecodeStyle::SmpteFull => {
            format!("{neg}{hours:02}:{minutes:02}:{seconds:02}:{frames:02}")
        }
    }
}

/// `BLI_timecode_string_from_time_simple` — `HH:MM:SS.ff` or `MM:SS.ff`.
pub fn timecode_string_from_time_simple(time_seconds: f64) -> String {
    let hr = time_seconds as i64 / 3600;
    let min = (time_seconds as i64 / 60) % 60;
    let sec = time_seconds as i64 % 60;
    let hun = (time_seconds % 1.0 * 100.0) as i64;
    if hr != 0 {
        format!("{hr:02}:{min:02}:{sec:02}.{hun:02}")
    } else {
        format!("{min:02}:{sec:02}.{hun:02}")
    }
}

/// `BLI_timecode_string_from_time_seconds`.
pub fn timecode_string_from_time_seconds(brevity_level: i32, time_seconds: f64) -> String {
    timecode_string_from_time(brevity_level, time_seconds as f32, 1.0, TimecodeStyle::Minimal)
}
