//! Port of `easing.cc` — all `BLI_easing_*` Robert Penner easing
//! functions (float time/begin/change/duration convention).

const M_PI: f32 = std::f32::consts::PI;
const M_PI_2: f32 = std::f32::consts::FRAC_PI_2;

/// `BLI_easing_back_ease_in`.
pub fn easing_back_ease_in(time: f32, begin: f32, change: f32, duration: f32, overshoot: f32) -> f32 {
    let time = time / duration;
    change * time * time * ((overshoot + 1.0) * time - overshoot) + begin
}
/// `BLI_easing_back_ease_out`.
pub fn easing_back_ease_out(time: f32, begin: f32, change: f32, duration: f32, overshoot: f32) -> f32 {
    let time = time / duration - 1.0;
    change * (time * time * ((overshoot + 1.0) * time + overshoot) + 1.0) + begin
}
/// `BLI_easing_back_ease_in_out`.
pub fn easing_back_ease_in_out(time: f32, begin: f32, change: f32, duration: f32, overshoot: f32) -> f32 {
    let overshoot = overshoot * 1.525;
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return change / 2.0 * (time * time * ((overshoot + 1.0) * time - overshoot)) + begin;
    }
    time -= 2.0;
    change / 2.0 * (time * time * ((overshoot + 1.0) * time + overshoot) + 2.0) + begin
}
/// `BLI_easing_bounce_ease_out`.
pub fn easing_bounce_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / duration;
    if time < 1.0 / 2.75 {
        return change * (7.5625 * time * time) + begin;
    }
    if time < 2.0 / 2.75 {
        time -= 1.5 / 2.75;
        return change * ((7.5625 * time) * time + 0.75) + begin;
    }
    if time < 2.5 / 2.75 {
        time -= 2.25 / 2.75;
        return change * ((7.5625 * time) * time + 0.9375) + begin;
    }
    time -= 2.625 / 2.75;
    change * ((7.5625 * time) * time + 0.984375) + begin
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
    easing_bounce_ease_out(time * 2.0 - duration, 0.0, change, duration) * 0.5
        + change * 0.5
        + begin
}
/// `BLI_easing_circ_ease_in`.
pub fn easing_circ_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    -change * ((1.0 - time * time).sqrt() - 1.0) + begin
}
/// `BLI_easing_circ_ease_out`.
pub fn easing_circ_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration - 1.0;
    change * (1.0 - time * time).sqrt() + begin
}
/// `BLI_easing_circ_ease_in_out`.
pub fn easing_circ_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return -change / 2.0 * ((1.0 - time * time).sqrt() - 1.0) + begin;
    }
    time -= 2.0;
    change / 2.0 * ((1.0 - time * time).sqrt() + 1.0) + begin
}
/// `BLI_easing_cubic_ease_in`.
pub fn easing_cubic_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    change * time * time * time + begin
}
/// `BLI_easing_cubic_ease_out`.
pub fn easing_cubic_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration - 1.0;
    change * (time * time * time + 1.0) + begin
}
/// `BLI_easing_cubic_ease_in_out`.
pub fn easing_cubic_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return change / 2.0 * time * time * time + begin;
    }
    time -= 2.0;
    change / 2.0 * (time * time * time + 2.0) + begin
}

fn elastic_blend(time: f32, change: f32, duration: f32, amplitude: f32, s: f32, f: f32) -> f32 {
    if change != 0.0 {
        let t = s.abs();
        let mut f = f;
        if amplitude != 0.0 {
            f *= amplitude / change.abs();
        } else {
            f = 0.0;
        }
        if (time * duration).abs() < t {
            let l = (time * duration).abs() / t;
            f = f * l + (1.0 - l);
        }
        return f;
    }
    f
}

/// `BLI_easing_elastic_ease_in`.
pub fn easing_elastic_ease_in(
    mut time: f32, begin: f32, change: f32, duration: f32,
    mut amplitude: f32, mut period: f32,
) -> f32 {
    let s: f32;
    let mut f = 1.0f32;
    if time == 0.0 {
        return begin;
    }
    time /= duration;
    if time == 1.0 {
        return begin + change;
    }
    time -= 1.0;
    if period == 0.0 {
        period = duration * 0.3;
    }
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * M_PI) * (change / amplitude).asin();
    }
    -f * (amplitude * (10.0 * time).exp2() * ((time * duration - s) * (2.0 * M_PI) / period).sin())
        + begin
}
/// `BLI_easing_elastic_ease_out`.
pub fn easing_elastic_ease_out(
    mut time: f32, begin: f32, change: f32, duration: f32,
    mut amplitude: f32, mut period: f32,
) -> f32 {
    let s: f32;
    let mut f = 1.0f32;
    if time == 0.0 {
        return begin;
    }
    time /= duration;
    if time == 1.0 {
        return begin + change;
    }
    time = -time;
    if period == 0.0 {
        period = duration * 0.3;
    }
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * M_PI) * (change / amplitude).asin();
    }
    f * (amplitude * (10.0 * time).exp2() * ((time * duration - s) * (2.0 * M_PI) / period).sin())
        + change
        + begin
}
/// `BLI_easing_elastic_ease_in_out`.
pub fn easing_elastic_ease_in_out(
    mut time: f32, begin: f32, change: f32, duration: f32,
    mut amplitude: f32, mut period: f32,
) -> f32 {
    let s: f32;
    let mut f = 1.0f32;
    if time == 0.0 {
        return begin;
    }
    time /= duration / 2.0;
    if time == 2.0 {
        return begin + change;
    }
    time -= 1.0;
    if period == 0.0 {
        period = duration * (0.3 * 1.5);
    }
    if amplitude == 0.0 || amplitude < change.abs() {
        s = period / 4.0;
        f = elastic_blend(time, change, duration, amplitude, s, f);
        amplitude = change;
    } else {
        s = period / (2.0 * M_PI) * (change / amplitude).asin();
    }
    if time < 0.0 {
        f *= -0.5;
        return f * (amplitude
            * (10.0 * time).exp2()
            * ((time * duration - s) * (2.0 * M_PI) / period).sin())
            + begin;
    }
    time = -time;
    f *= 0.5;
    f * (amplitude * (10.0 * time).exp2() * ((time * duration - s) * (2.0 * M_PI) / period).sin())
        + change
        + begin
}

const POW_MIN: f32 = 0.0009765625;
const POW_SCALE: f32 = 1.0 / (1.0 - 0.0009765625);

/// `BLI_easing_expo_ease_in`.
pub fn easing_expo_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    if time == 0.0 {
        return begin;
    }
    change * ((10.0 * (time / duration - 1.0)).exp2() - POW_MIN) * POW_SCALE + begin
}
/// `BLI_easing_expo_ease_out`.
pub fn easing_expo_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    if time == 0.0 {
        return begin;
    }
    change * (1.0 - ((-10.0 * time / duration).exp2() - POW_MIN) * POW_SCALE) + begin
}
/// `BLI_easing_expo_ease_in_out`.
pub fn easing_expo_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let duration_half = duration / 2.0;
    let change_half = change / 2.0;
    if time <= duration_half {
        return easing_expo_ease_in(time, begin, change_half, duration_half);
    }
    easing_expo_ease_out(time - duration_half, begin + change_half, change_half, duration_half)
}
/// `BLI_easing_linear_ease`.
pub fn easing_linear_ease(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    change * time / duration + begin
}
/// `BLI_easing_quad_ease_in`.
pub fn easing_quad_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    change * time * time + begin
}
/// `BLI_easing_quad_ease_out`.
pub fn easing_quad_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    -change * time * (time - 2.0) + begin
}
/// `BLI_easing_quad_ease_in_out`.
pub fn easing_quad_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return change / 2.0 * time * time + begin;
    }
    time -= 1.0;
    -change / 2.0 * (time * (time - 2.0) - 1.0) + begin
}
/// `BLI_easing_quart_ease_in`.
pub fn easing_quart_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    change * time * time * time * time + begin
}
/// `BLI_easing_quart_ease_out`.
pub fn easing_quart_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration - 1.0;
    -change * (time * time * time * time - 1.0) + begin
}
/// `BLI_easing_quart_ease_in_out`.
pub fn easing_quart_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return change / 2.0 * time * time * time * time + begin;
    }
    time -= 2.0;
    -change / 2.0 * (time * time * time * time - 2.0) + begin
}
/// `BLI_easing_quint_ease_in`.
pub fn easing_quint_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration;
    change * time * time * time * time * time + begin
}
/// `BLI_easing_quint_ease_out`.
pub fn easing_quint_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let time = time / duration - 1.0;
    change * (time * time * time * time * time + 1.0) + begin
}
/// `BLI_easing_quint_ease_in_out`.
pub fn easing_quint_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    let mut time = time / (duration / 2.0);
    if time < 1.0 {
        return change / 2.0 * time * time * time * time * time + begin;
    }
    time -= 2.0;
    change / 2.0 * (time * time * time * time * time + 2.0) + begin
}
/// `BLI_easing_sine_ease_in`.
pub fn easing_sine_ease_in(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    -change * (time / duration * M_PI_2).cos() + change + begin
}
/// `BLI_easing_sine_ease_out`.
pub fn easing_sine_ease_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    change * (time / duration * M_PI_2).sin() + begin
}
/// `BLI_easing_sine_ease_in_out`.
pub fn easing_sine_ease_in_out(time: f32, begin: f32, change: f32, duration: f32) -> f32 {
    -change / 2.0 * (M_PI * time / duration).cos() + change / 2.0 + begin
}
