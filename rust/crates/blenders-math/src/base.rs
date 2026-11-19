//! Port of `BLI_math_base.hh` / `BLI_math_base_safe.hh`.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// `safe_acosf` — acos clamped to [-1, 1].
#[inline]
pub fn safe_acos(a: f32) -> f32 {
    if a <= -1.0 {
        PI
    } else if a >= 1.0 {
        0.0
    } else {
        a.acos()
    }
}

/// `safe_sqrtf` — sqrt of a value clamped to >= 0.
#[inline]
pub fn safe_sqrt(a: f32) -> f32 {
    if a <= 0.0 { 0.0 } else { a.sqrt() }
}

/// `safe_asinf`.
#[inline]
pub fn safe_asin(a: f32) -> f32 {
    if a <= -1.0 {
        -FRAC_PI_2
    } else if a >= 1.0 {
        FRAC_PI_2
    } else {
        a.asin()
    }
}

/// `signf`.
#[inline]
pub fn sign(a: f32) -> f32 {
    if a < 0.0 { -1.0 } else { 1.0 }
}

/// `square_f`.
#[inline]
pub fn square(a: f32) -> f32 {
    a * a
}

/// `pow2f`.
#[inline]
pub fn pow2(a: f32) -> f32 {
    a * a
}

/// `hypotf` wrapper.
#[inline]
pub fn hypot(x: f32, y: f32) -> f32 {
    x.hypot(y)
}

/// `wrap_f` — wrap `value` into `[min, max)`.
#[inline]
pub fn wrap(value: f32, min: f32, max: f32) -> f32 {
    let range = max - min;
    if range == 0.0 {
        return max;
    }
    let v = (value - min) % range;
    min + if v < 0.0 { v + range } else { v }
}

/// Wrap an angle to `[-pi, pi)` (`AngleRadianBase::wrapped`).
#[inline]
pub fn wrap_angle(angle: f32) -> f32 {
    wrap(angle, -PI, PI)
}

/// `interpf` — linear interpolation.
#[inline]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// `compare_ff_relative` — relative epsilon comparison.
#[inline]
pub fn compare_ff_relative(a: f32, b: f32, max_diff: f32, max_rel_diff: i32) -> bool {
    let diff = (a - b).abs();
    if diff <= max_diff {
        return true;
    }
    let relative = if a.abs() < b.abs() { b } else { a };
    diff <= relative.abs() * max_rel_diff as f32 * f32::EPSILON
}

/// `is_finite` for floats.
#[inline]
pub fn is_finite(a: f32) -> bool {
    a.is_finite()
}

/// Mod floored (`floored_fmod` behaviour for positive divisor semantics of `mod_ff`).
#[inline]
pub fn mod_floor(a: f32, b: f32) -> f32 {
    a - (a / b).floor() * b
}

/// `saacosf` alias kept for C-name parity.
#[inline]
pub fn saacos(a: f32) -> f32 {
    safe_acos(a)
}

/// 2π.
pub const M_TAU: f32 = TAU;
/// π.
pub const M_PI: f32 = PI;
/// π/2.
pub const M_PI_2: f32 = FRAC_PI_2;
/// √2.
pub const M_SQRT2: f32 = std::f32::consts::SQRT_2;

/// `FLT_EPSILON` from BLI_math_base_c.
pub const FLT_EPSILON: f32 = f32::EPSILON;

/// `min_ff`.
#[inline]
pub fn min_ff(a: f32, b: f32) -> f32 {
    a.min(b)
}

/// `max_ff`.
#[inline]
pub fn max_ff(a: f32, b: f32) -> f32 {
    a.max(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_acos_clamps() {
        assert_eq!(safe_acos(2.0), 0.0);
        assert_eq!(safe_acos(-2.0), PI);
        assert_eq!(safe_acos(0.0), FRAC_PI_2);
    }

    #[test]
    fn wrap_cycles() {
        assert!((wrap(370.0, 0.0, 360.0) - 10.0).abs() < 1e-6);
        assert!((wrap(-10.0, 0.0, 360.0) - 350.0).abs() < 1e-6);
    }
}
