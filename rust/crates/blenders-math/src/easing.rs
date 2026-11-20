//! Port of `easing.cc` — Robert Penner easing functions (BSD-3).
//! Signatures: `(time, begin, change, duration, ...)`.

use std::f32::consts::{FRAC_PI_2, PI};

/// `BLI_easing_back_ease_in`.
pub fn easing_back_ease_in(time: f32, begin: f32, change: f32, duration: f32, overshoot: f32) -> f32 {
    let t = time / duration;
    change * t * t * ((overshoot + 1.0) * t - overshoot) + begin
}
/// `BLI_easing_back_ease_out`.
pub fn easing_back_ease_out(time: f32, begin: f32, change: f32, duration: f32, overshoot: f32) -> f32 {
    let t = time / duration - 1.0;
    change * (t * t * ((overshoot + 1.0) * t + overshoot) + 1.0) + begin
}
/// `BLI_easing_back_ease_in_out`.
pub fn easing_back_ease_in_out(
    time: f32, begin: f32, change: f32, duration: f32, overshoot: f32,
) -> f32 {
    let overshoot = overshoot * 1.525;
    let t = time / (duration / 2.0);
    if t < 1.0 {
        return change / 2.0 * (t * t * ((overshoot + 1.0) * t - overshoot)) + begin;
    }
    let t = t - 2.0;
    change / 2.0 * (t * t * ((overshoot + 1.0) * t + overshoot) + 2.0) + begin
}
/// `BLI_easing_bounce_ease_out`.
pub fn easing_bounce_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / duration;
    if t < 1.0 / 2.75 {
        return change * (7.5625 * t * t) + begin;
    }
    if t < 2.0 / 2.75 {
        t -= 1.5 / 2.75;
        return change * ((7.5625 * t) * t + 0.75) + begin;
    }
    if t < 2.5 / 2.75 {
        t -= 2.25 / 2.75;
        return change * ((7.5625 * t) * t + 0.9375) + begin;
    }
    t -= 2.625 / 2.75;
    change * ((7.5625 * t) * t + 0.984375) + begin
}
/// `BLI_easing_bounce_ease_in`.
pub fn easing_bounce_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    change - easing_bounce_ease_out(duration - time, 0.0, change, duration) + begin
}
/// `BLI_easing_bounce_ease_in_out`.
pub fn easing_bounce_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    if time < duration / 2.0 {
        return easing_bounce_ease_in(time * 2.0, 0.0, change, duration) * 0.5 + begin;
    }
    easing_bounce_ease_out(time * 2.0 - duration, 0.0, change, duration) * 0.5 + change * 0.5 + begin
}
/// `BLI_easing_circ_ease_in`.
pub fn easing_circ_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    -change * ((1.0 - t * t).sqrt() - 1.0) + begin
}
/// `BLI_easing_circ_ease_out`.
pub fn easing_circ_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration - 1.0;
    change * (1.0 - t * t).sqrt() + begin
}
/// `BLI_easing_circ_ease_in_out`.
pub fn easing_circ_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / (duration / 2.0);
    if t < 1.0 {
        return -change / 2.0 * ((1.0 - t * t).sqrt() - 1.0) + begin;
    }
    t -= 2.0;
    change / 2.0 * ((1.0 - t * t).sqrt() + 1.0) + begin
}
/// `BLI_easing_cubic_ease_in`.
pub fn easing_cubic_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    change * t * t * t + begin
}
/// `BLI_easing_cubic_ease_out`.
pub fn easing_cubic_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration - 1.0;
    change * (t * t * t + 1.0) + begin
}
/// `BLI_easing_cubic_ease_in_out`.
pub fn easing_cubic_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / (duration / 2.0);
    if t < 1.0 {
        return change / 2.0 * t * t * t + begin;
    }
    t -= 2.0;
    change / 2.0 * (t * t * t + 2.0) + begin
}

/// `elastic_blend` (USE_ELASTIC_BLEND).
fn elastic_blend(time: f32, change: f32, duration: f32, amplitude: f32, s: f32, mut f: f32) -> f32 {
    if change != 0.0 {
        let t = s.abs();
        f = if amplitude != 0.0 {
            f * amplitude / change.abs()
        } else {
            0.0
        };
        if (time * duration).abs() < t {
            let l = (time * duration).abs() / t;
            f = (f * l) + (1.0 - l);
        }
    }
    f
}

/// `BLI_easing_elastic_ease_in`.
pub fn easing_elastic_ease_in(
    mut time: f32, begin: f32, change: f32, duration: f32, amplitude: f32, period: f32,
) -> f32 {
    let mut f = 1.0;
    if time == 0.0 {
        return begin;
    }
    time /= duration;
    if time == 1.0 {
        return begin + change;
    }
    time -= 1.0;
    let mut period = period;
    if period == 0.0 {
        period = duration * 0.3;
    }
    let mut amplitude = amplitude;
    let s;
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * PI) * (change / amplitude).asin();
    }
    (-f * (amplitude * 2f32.powf(10.0 * time) * ((time * duration - s) * (2.0 * PI) / period).sin()))
        + begin
}
/// `BLI_easing_elastic_ease_out`.
pub fn easing_elastic_ease_out(
    mut time: f32, begin: f32, change: f32, duration: f32, amplitude: f32, period: f32,
) -> f32 {
    let mut f = 1.0;
    if time == 0.0 {
        return begin;
    }
    time /= duration;
    if time == 1.0 {
        return begin + change;
    }
    time = -time;
    let mut period = period;
    if period == 0.0 {
        period = duration * 0.3;
    }
    let mut amplitude = amplitude;
    let s;
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * PI) * (change / amplitude).asin();
    }
    (f * (amplitude * 2f32.powf(10.0 * time) * ((time * duration - s) * (2.0 * PI) / period).sin()))
        + change
        + begin
}
/// `BLI_easing_elastic_ease_in_out`.
pub fn easing_elastic_ease_in_out(
    mut time: f32, begin: f32, change: f32, duration: f32, amplitude: f32, period: f32,
) -> f32 {
    let mut f = 1.0;
    if time == 0.0 {
        return begin;
    }
    time /= duration / 2.0;
    if time == 2.0 {
        return begin + change;
    }
    time -= 1.0;
    let mut period = period;
    if period == 0.0 {
        period = duration * (0.3 * 1.5);
    }
    let mut amplitude = amplitude;
    let s;
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * PI) * (change / amplitude).asin();
    }
    if time < 0.0 {
        f *= -0.5;
        return (f
            * (amplitude
                * 2f32.powf(10.0 * time)
                * ((time * duration - s) * (2.0 * PI) / period).sin()))
            + begin;
    }
    time = -time;
    f *= 0.5;
    (f * (amplitude * 2f32.powf(10.0 * time) * ((time * duration - s) * (2.0 * PI) / period).sin()))
        + change
        + begin
}

const POW_MIN: f32 = 0.0009765625; /* 2^-10 */
const POW_SCALE: f32 = 1.0 / (1.0 - 0.0009765625);

/// `BLI_easing_expo_ease_in`.
pub fn easing_expo_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    if time == 0.0 {
        return begin;
    }
    change * (2f32.powf(10.0 * (time / duration - 1.0)) - POW_MIN) * POW_SCALE + begin
}
/// `BLI_easing_expo_ease_out`.
pub fn easing_expo_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    if time == 0.0 {
        return begin;
    }
    change * (1.0 - (2f32.powf(-10.0 * time / duration) - POW_MIN) * POW_SCALE) + begin
}
/// `BLI_easing_expo_ease_in_out`.
pub fn easing_expo_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let dh = duration / 2.0;
    let ch = change / 2.0;
    if time <= dh {
        return easing_expo_ease_in(time, begin, ch, dh);
    }
    easing_expo_ease_out(time - dh, begin + ch, ch, dh)
}
/// `BLI_easing_linear_ease`.
pub fn easing_linear_ease(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    change * time / duration + begin
}
/// `BLI_easing_quad_ease_in`.
pub fn easing_quad_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    change * t * t + begin
}
/// `BLI_easing_quad_ease_out`.
pub fn easing_quad_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    -change * t * (t - 2.0) + begin
}
/// `BLI_easing_quad_ease_in_out`.
pub fn easing_quad_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / (duration / 2.0);
    if t < 1.0 {
        return change / 2.0 * t * t + begin;
    }
    t -= 1.0;
    -change / 2.0 * (t * (t - 2.0) - 1.0) + begin
}
/// `BLI_easing_quart_ease_in`.
pub fn easing_quart_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    change * t * t * t * t + begin
}
/// `BLI_easing_quart_ease_out`.
pub fn easing_quart_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration - 1.0;
    -change * (t * t * t * t - 1.0) + begin
}
/// `BLI_easing_quart_ease_in_out`.
pub fn easing_quart_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / (duration / 2.0);
    if t < 1.0 {
        return change / 2.0 * t * t * t * t + begin;
    }
    t -= 2.0;
    -change / 2.0 * (t * t * t * t - 2.0) + begin
}
/// `BLI_easing_quint_ease_in`.
pub fn easing_quint_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration;
    change * t * t * t * t * t + begin
}
/// `BLI_easing_quint_ease_out`.
pub fn easing_quint_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let t = time / duration - 1.0;
    change * (t * t * t * t * t + 1.0) + begin
}
/// `BLI_easing_quint_ease_in_out`.
pub fn easing_quint_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut t = time / (duration / 2.0);
    if t < 1.0 {
        return change / 2.0 * t * t * t * t * t + begin;
    }
    t -= 2.0;
    change / 2.0 * (t * t * t * t * t + 2.0) + begin
}
/// `BLI_easing_sine_ease_in`.
pub fn easing_sine_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    -change * (time / duration * FRAC_PI_2).cos() + change + begin
}
/// `BLI_easing_sine_ease_out`.
pub fn easing_sine_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    change * (time / duration * FRAC_PI_2).sin() + begin
}
/// `BLI_easing_sine_ease_in_out`.
pub fn easing_sine_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    -change / 2.0 * ((PI * time / duration).cos() - 1.0) + begin
}
