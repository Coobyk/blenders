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

/* ---- math_base.cc ---- */

/// `pow_i` — integer power by squaring.
pub fn pow_i(mut base: i32, mut exp: i32) -> i32 {
    debug_assert!(exp >= 0);
    let mut result = 1;
    while exp != 0 {
        if exp & 1 != 0 {
            result *= base;
        }
        exp >>= 1;
        base *= base;
    }
    result
}

/// `double_round` — round to `ndigits` decimal places, half-even.
pub fn double_round(x: f64, ndigits: i32) -> f64 {
    let (pow1, y);
    if ndigits >= 0 {
        pow1 = 10f64.powi(ndigits);
        y = x * pow1;
        if !y.is_finite() {
            return x;
        }
    } else {
        pow1 = 10f64.powi(-ndigits);
        y = x / pow1;
    }
    let mut z = y.round();
    if (y - z).abs() == 0.5 {
        z = 2.0 * (y / 2.0).round();
    }
    if ndigits >= 0 {
        z / pow1
    } else {
        z * pow1
    }
}

/// `floor_power_of_10`.
pub fn floor_power_of_10(f: f32) -> f32 {
    debug_assert!(f >= 0.0);
    if f != 0.0 {
        1.0 / 10f32.powf((1.0 / f).log10().ceil())
    } else {
        0.0
    }
}

/// `ceil_power_of_10`.
pub fn ceil_power_of_10(f: f32) -> f32 {
    debug_assert!(f >= 0.0);
    if f != 0.0 {
        1.0 / 10f32.powf((1.0 / f).log10().floor())
    } else {
        0.0
    }
}

/* ---- math_time.cc ---- */

pub const SECONDS_IN_DAY: f64 = 86400.0;
pub const SECONDS_IN_HOUR: f64 = 3600.0;
pub const SECONDS_IN_MINUTE: f64 = 60.0;
pub const SECONDS_IN_MILLISECONDS: f64 = 1.0e-3;

/// `BLI_math_time_seconds_decompose` — decompose seconds into requested
/// components. Pass `Some(&mut f)` for each part wanted; remainder rolls into
/// the last requested component.
pub fn math_time_seconds_decompose(
    mut seconds: f64,
    r_days: Option<&mut f64>,
    r_hours: Option<&mut f64>,
    r_minutes: Option<&mut f64>,
    r_seconds: Option<&mut f64>,
    r_milliseconds: Option<&mut f64>,
) {
    let mut r_days = r_days;
    let mut r_hours = r_hours;
    let mut r_minutes = r_minutes;
    let mut r_seconds = r_seconds;
    if let Some(d) = r_days.as_deref_mut() {
        let (frac, whole) = {
            let v = seconds / SECONDS_IN_DAY;
            (v.fract(), v.trunc())
        };
        *d = whole;
        seconds = frac * SECONDS_IN_DAY;
    }
    if let Some(h) = r_hours.as_deref_mut() {
        let v = seconds / SECONDS_IN_HOUR;
        *h = v.trunc();
        seconds = v.fract() * SECONDS_IN_HOUR;
    }
    if let Some(m) = r_minutes.as_deref_mut() {
        let v = seconds / SECONDS_IN_MINUTE;
        *m = v.trunc();
        seconds = v.fract() * SECONDS_IN_MINUTE;
    }
    if let Some(s) = r_seconds.as_deref_mut() {
        *s = seconds.trunc();
        seconds = seconds.fract();
    }
    if let Some(ms) = r_milliseconds {
        *ms = seconds / SECONDS_IN_MILLISECONDS;
    } else if let Some(s) = r_seconds.as_deref_mut() {
        *s += seconds;
    } else if let Some(m) = r_minutes.as_deref_mut() {
        *m += seconds / SECONDS_IN_MINUTE;
    } else if let Some(h) = r_hours.as_deref_mut() {
        *h += seconds / SECONDS_IN_HOUR;
    } else if let Some(d) = r_days {
        *d = seconds / SECONDS_IN_DAY;
    }
}

/* ---- math_base_safe_inline.cc ---- */

/// `safe_divide` — 0 when denominator is 0.
#[inline]
pub fn safe_divide(a: f32, b: f32) -> f32 {
    if b != 0.0 {
        a / b
    } else {
        0.0
    }
}

/// `safe_modf`.
#[inline]
pub fn safe_modf(a: f32, b: f32) -> f32 {
    if b != 0.0 {
        a % b
    } else {
        0.0
    }
}

/// `safe_floored_modf`.
#[inline]
pub fn safe_floored_modf(a: f32, b: f32) -> f32 {
    if b != 0.0 {
        a - (a / b).floor() * b
    } else {
        0.0
    }
}

/// `safe_logf` — 0 on non-positive inputs.
#[inline]
pub fn safe_logf(a: f32, base: f32) -> f32 {
    if a <= 0.0 || base <= 0.0 {
        return 0.0;
    }
    safe_divide(a.ln(), base.ln())
}

/// `safe_inverse_sqrtf`.
#[inline]
pub fn safe_inverse_sqrtf(a: f32) -> f32 {
    if a > 0.0 {
        1.0 / a.sqrt()
    } else {
        0.0
    }
}

/// `safe_powf` — 0 for negative base with fractional exponent.
#[inline]
pub fn safe_powf(base: f32, exponent: f32) -> f32 {
    if base < 0.0 && exponent != (exponent as i32) as f32 {
        return 0.0;
    }
    base.powf(exponent)
}

/* ---- math_bits_inline.cc ---- */

/// `bitscan_forward_uint` — index of lowest set bit.
#[inline]
pub fn bitscan_forward_uint(a: u32) -> u32 {
    debug_assert!(a != 0);
    a.trailing_zeros()
}

/// `bitscan_forward_uint64`.
#[inline]
pub fn bitscan_forward_uint64(a: u64) -> u32 {
    debug_assert!(a != 0);
    a.trailing_zeros()
}

/// `bitscan_forward_clear_uint` — index of lowest set bit; clears it.
#[inline]
pub fn bitscan_forward_clear_uint(a: &mut u32) -> u32 {
    let i = bitscan_forward_uint(*a);
    *a &= *a - 1;
    i
}

/// `bitscan_forward_clear_uint64`.
#[inline]
pub fn bitscan_forward_clear_uint64(a: &mut u64) -> u32 {
    let i = bitscan_forward_uint64(*a);
    *a &= *a - 1;
    i
}

/// `bitscan_reverse_uint` — index of highest set bit.
#[inline]
pub fn bitscan_reverse_uint(a: u32) -> u32 {
    debug_assert!(a != 0);
    31 - a.leading_zeros()
}

/// `bitscan_reverse_uint64`.
#[inline]
pub fn bitscan_reverse_uint64(a: u64) -> u32 {
    debug_assert!(a != 0);
    63 - a.leading_zeros()
}

/// `bitscan_reverse_clear_uint`.
#[inline]
pub fn bitscan_reverse_clear_uint(a: &mut u32) -> u32 {
    let i = bitscan_reverse_uint(*a);
    *a &= !(1u32 << i);
    i
}

/// `highest_order_bit_uint` — the highest set bit as a power of two.
#[inline]
pub fn highest_order_bit_uint(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    1u32 << (31 - n.leading_zeros())
}

/// `highest_order_bit_s`.
#[inline]
pub fn highest_order_bit_s(n: u16) -> u16 {
    if n == 0 {
        return 0;
    }
    1u16 << (15 - n.leading_zeros())
}

/// `count_bits_i` — popcount.
#[inline]
pub fn count_bits_i(i: u32) -> i32 {
    i.count_ones() as i32
}

/// `count_bits_uint64`.
#[inline]
pub fn count_bits_uint64(a: u64) -> i32 {
    a.count_ones() as i32
}

/// `float_as_int`.
#[inline]
pub fn float_as_int(f: f32) -> i32 {
    f.to_bits() as i32
}

/// `float_as_uint`.
#[inline]
pub fn float_as_uint(f: f32) -> u32 {
    f.to_bits()
}

/// `int_as_float`.
#[inline]
pub fn int_as_float(i: i32) -> f32 {
    f32::from_bits(i as u32)
}

/// `uint_as_float`.
#[inline]
pub fn uint_as_float(i: u32) -> f32 {
    f32::from_bits(i)
}

/// `xor_fl` — flip bits of `x` where `y`'s bits are set.
#[inline]
pub fn xor_fl(x: f32, y: i32) -> f32 {
    int_as_float(float_as_int(x) ^ y)
}

/* ---- math_base_inline.cc ---- */

#[inline]
pub fn pow2f(x: f32) -> f32 {
    x * x
}
#[inline]
pub fn pow3f(x: f32) -> f32 {
    pow2f(x) * x
}
#[inline]
pub fn pow4f(x: f32) -> f32 {
    pow2f(pow2f(x))
}
#[inline]
pub fn pow5f(x: f32) -> f32 {
    pow4f(x) * x
}
#[inline]
pub fn pow7f(x: f32) -> f32 {
    pow2f(pow3f(x)) * x
}

/// `sqrt3f` — signed cube root.
#[inline]
pub fn sqrt3f(f: f32) -> f32 {
    if f == 0.0 {
        return 0.0;
    }
    if f < 0.0 {
        return -(f32::exp(f32::ln(-f) / 3.0));
    }
    f32::exp(f32::ln(f) / 3.0)
}

/// `sqrt3d`.
#[inline]
pub fn sqrt3d(d: f64) -> f64 {
    if d == 0.0 {
        return 0.0;
    }
    if d < 0.0 {
        return -f64::exp(f64::ln(-d) / 3.0);
    }
    f64::exp(f64::ln(d) / 3.0)
}

/// `sqrtf_signed`.
#[inline]
pub fn sqrtf_signed(f: f32) -> f32 {
    if f >= 0.0 {
        f.sqrt()
    } else {
        -(-f).sqrt()
    }
}

/// `interpf`.
#[inline]
pub fn interpf(target: f32, origin: f32, fac: f32) -> f32 {
    (fac * target) + (1.0 - fac) * origin
}

/// `interpd`.
#[inline]
pub fn interpd(target: f64, origin: f64, fac: f64) -> f64 {
    (fac * target) + (1.0 - fac) * origin
}

/// `ratiof`.
#[inline]
pub fn ratiof(min: f32, max: f32, pos: f32) -> f32 {
    let range = max - min;
    if range == 0.0 {
        0.0
    } else {
        (pos - min) / range
    }
}

/// `ratiod`.
#[inline]
pub fn ratiod(min: f64, max: f64, pos: f64) -> f64 {
    let range = max - min;
    if range == 0.0 {
        0.0
    } else {
        (pos - min) / range
    }
}

/// `power_of_2` — smallest power of two >= val.
#[inline]
pub fn power_of_2(val: f32) -> f32 {
    2f64.powf((f64::from(val).ln() / std::f64::consts::LN_2).ceil()) as f32
}

/// `is_power_of_2_i`.
#[inline]
pub fn is_power_of_2_i(n: i32) -> bool {
    (n & (n - 1)) == 0
}

/// `power_of_2_max_i` — smallest power of two > n (or n if already one).
#[inline]
pub fn power_of_2_max_i(mut n: i32) -> i32 {
    if is_power_of_2_i(n) {
        return n;
    }
    loop {
        n &= n - 1;
        if is_power_of_2_i(n) {
            break;
        }
    }
    n * 2
}

/// `power_of_2_min_i` — largest power of two <= n.
#[inline]
pub fn power_of_2_min_i(mut n: i32) -> i32 {
    while !is_power_of_2_i(n) {
        n &= n - 1;
    }
    n
}

/// `power_of_2_max_u` — next power of two (bit trick).
#[inline]
pub fn power_of_2_max_u(mut x: u32) -> u32 {
    x -= 1;
    x |= x >> 1;
    x |= x >> 2;
    x |= x >> 4;
    x |= x >> 8;
    x |= x >> 16;
    x + 1
}

/// `log2_floor_u`.
#[inline]
pub fn log2_floor_u(x: u32) -> u32 {
    if x <= 1 {
        0
    } else {
        31 - x.leading_zeros()
    }
}

/// `log2_ceil_u`.
#[inline]
pub fn log2_ceil_u(x: u32) -> u32 {
    if is_power_of_2_i(x as i32) {
        log2_floor_u(x)
    } else {
        log2_floor_u(x) + 1
    }
}

/* rounding and clamping */

#[inline]
pub fn round_fl_to_uchar(a: f32) -> u8 {
    (a + 0.5).floor() as u8
}
#[inline]
pub fn round_fl_to_short(a: f32) -> i16 {
    (a + 0.5).floor() as i16
}
#[inline]
pub fn round_fl_to_int(a: f32) -> i32 {
    (a + 0.5).floor() as i32
}
#[inline]
pub fn round_fl_to_uint(a: f32) -> u32 {
    (a + 0.5).floor() as u32
}
#[inline]
pub fn round_db_to_int(a: f64) -> i32 {
    (a + 0.5).floor() as i32
}
#[inline]
pub fn round_fl_to_uchar_clamp(a: f32) -> u8 {
    let r = (a + 0.5).floor();
    if r <= 0.0 {
        0
    } else if r >= u8::MAX as f32 {
        u8::MAX
    } else {
        r as u8
    }
}
#[inline]
pub fn round_fl_to_int_clamp(a: f32) -> i32 {
    let r = (a + 0.5).floor();
    if r <= i32::MIN as f32 {
        i32::MIN
    } else if r >= i32::MAX as f32 {
        i32::MAX
    } else {
        r as i32
    }
}
#[inline]
pub fn round_db_to_uchar_clamp(a: f64) -> u8 {
    let r = (a + 0.5).floor();
    if r <= 0.0 {
        0
    } else if r >= u8::MAX as f64 {
        u8::MAX
    } else {
        r as u8
    }
}
#[inline]
pub fn round_db_to_short_clamp(a: f64) -> i16 {
    let r = (a + 0.5).floor();
    if r <= i16::MIN as f64 {
        i16::MIN
    } else if r >= i16::MAX as f64 {
        i16::MAX
    } else {
        r as i16
    }
}
#[inline]
pub fn round_db_to_int_clamp(a: f64) -> i32 {
    let r = (a + 0.5).floor();
    if r <= i32::MIN as f64 {
        i32::MIN
    } else if r >= i32::MAX as f64 {
        i32::MAX
    } else {
        r as i32
    }
}

/// `round_to_even`.
#[inline]
pub fn round_to_even(f: f32) -> f32 {
    (f * 0.5).round() * 2.0
}

/// `divide_round_i`.
#[inline]
pub fn divide_round_i(a: i32, b: i32) -> i32 {
    (2 * a + b) / (2 * b)
}

/// `divide_floor_i` — floors negative results (Python-style).
#[inline]
pub fn divide_floor_i(a: i32, b: i32) -> i32 {
    let d = a / b;
    let r = a % b;
    if r != 0 {
        d - (((a < 0) != (b < 0)) as i32)
    } else {
        d
    }
}

/// `divide_ceil_u`.
#[inline]
pub fn divide_ceil_u(a: u32, b: u32) -> u32 {
    (a + b - 1) / b
}

/// `divide_ceil_ul`.
#[inline]
pub fn divide_ceil_ul(a: u64, b: u64) -> u64 {
    (a + b - 1) / b
}

/// `ceil_to_multiple_u`.
#[inline]
pub fn ceil_to_multiple_u(a: u32, b: u32) -> u32 {
    divide_ceil_u(a, b) * b
}

/// `ceil_to_multiple_ul`.
#[inline]
pub fn ceil_to_multiple_ul(a: u64, b: u64) -> u64 {
    divide_ceil_ul(a, b) * b
}

/// `mod_i` — positive remainder.
#[inline]
pub fn mod_i(i: i32, n: i32) -> i32 {
    (i % n + n) % n
}

/// `floored_fmod`.
#[inline]
pub fn floored_fmod(f: f32, n: f32) -> f32 {
    f - n * (f / n).floor()
}

/// `fractf`.
#[inline]
pub fn fractf(a: f32) -> f32 {
    a - a.floor()
}

/// `wrapf` — wrap value into [min, max).
#[inline]
pub fn wrapf(value: f32, max: f32, min: f32) -> f32 {
    let range = max - min;
    if range != 0.0 {
        value - range * ((value - min) / range).floor()
    } else {
        min
    }
}

/// `pingpongf`.
#[inline]
pub fn pingpongf(value: f32, scale: f32) -> f32 {
    if scale == 0.0 {
        return 0.0;
    }
    (fractf((value - scale) / (scale * 2.0)) * scale * 2.0 - scale).abs()
}

#[inline]
pub fn square_s(a: i16) -> i32 {
    a as i32 * a as i32
}
#[inline]
pub fn square_i(a: i32) -> i32 {
    a * a
}
#[inline]
pub fn square_uint(a: u32) -> u32 {
    a * a
}
#[inline]
pub fn square_f(a: f32) -> f32 {
    a * a
}
#[inline]
pub fn cube_i(a: i32) -> i32 {
    a * a * a
}
#[inline]
pub fn cube_f(a: f32) -> f32 {
    a * a * a
}

#[inline]
pub fn min_ff(a: f32, b: f32) -> f32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_ff(a: f32, b: f32) -> f32 {
    if a > b {
        a
    } else {
        b
    }
}

/// `smoothminf` — iq's smooth minimum.
#[inline]
pub fn smoothminf(a: f32, b: f32, c: f32) -> f32 {
    if c != 0.0 {
        let h = max_ff(c - (a - b).abs(), 0.0) / c;
        return min_ff(a, b) - h * h * h * c * (1.0 / 6.0);
    }
    min_ff(a, b)
}

/// `smoothstep`.
#[inline]
pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if x < edge0 {
        0.0
    } else if x >= edge1 {
        1.0
    } else {
        let t = (x - edge0) / (edge1 - edge0);
        (3.0 - 2.0 * t) * (t * t)
    }
}

#[inline]
pub fn min_dd(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_dd(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}
#[inline]
pub fn min_ii(a: i32, b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_ii(a: i32, b: i32) -> i32 {
    if b < a {
        a
    } else {
        b
    }
}
#[inline]
pub fn min_uu(a: u32, b: u32) -> u32 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_uu(a: u32, b: u32) -> u32 {
    if b < a {
        a
    } else {
        b
    }
}
#[inline]
pub fn min_ulul(a: u64, b: u64) -> u64 {
    if a < b {
        a
    } else {
        b
    }
}
#[inline]
pub fn max_ulul(a: u64, b: u64) -> u64 {
    if b < a {
        a
    } else {
        b
    }
}

/// `clamp_i`.
#[inline]
pub fn clamp_i(value: i32, min: i32, max: i32) -> i32 {
    min_ii(max_ii(value, min), max)
}

/// `clamp_f`.
#[inline]
pub fn clamp_f(value: f32, min: f32, max: f32) -> f32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// `compare_ff` — |a-b| <= max_diff.
#[inline]
pub fn compare_ff(a: f32, b: f32, max_diff: f32) -> bool {
    (a - b).abs() <= max_diff
}

/// `ulp_diff_ff` — ULP distance between two floats.
#[inline]
pub fn ulp_diff_ff(a: f32, b: f32) -> u32 {
    const SIGN_BIT: u32 = 0x8000_0000;
    const INFINITY: u32 = 0x7f80_0000;
    let (ai, bi) = (a.to_bits(), b.to_bits());
    let (a_sign, b_sign) = (ai & SIGN_BIT, bi & SIGN_BIT);
    let (a_abs, b_abs) = (ai & !SIGN_BIT, bi & !SIGN_BIT);
    if a_abs > INFINITY || b_abs > INFINITY {
        return 0xffff_ffff;
    }
    if a_sign == b_sign {
        a_abs.max(b_abs) - a_abs.min(b_abs)
    } else {
        a_abs + b_abs
    }
}

/// `compare_ff_relative`.
#[inline]
pub fn compare_ff_relative(a: f32, b: f32, max_diff: f32, max_ulps: i32) -> bool {
    debug_assert!(max_ulps >= 0 && max_ulps < (1 << 22));
    if (a - b).abs() <= max_diff {
        return true;
    }
    ulp_diff_ff(a, b) <= max_ulps as u32
}

/// `compare_threshold_relative`.
#[inline]
pub fn compare_threshold_relative(value1: f32, value2: f32, thresh: f32) -> bool {
    let abs_diff = (value1 - value2).abs();
    if value2.abs() < 1.0 {
        return abs_diff > thresh;
    }
    abs_diff > thresh * value2.abs()
}

/// `increment_ulp` — next representable float.
#[inline]
pub fn increment_ulp(value: f32) -> f32 {
    if !value.is_finite() {
        return value;
    }
    let mut i = value.to_bits();
    if value > 0.0 {
        i += 1;
    } else if value < -0.0 {
        i -= 1;
    } else {
        i = 0x0000_0001;
    }
    f32::from_bits(i)
}

/// `decrement_ulp`.
#[inline]
pub fn decrement_ulp(value: f32) -> f32 {
    if !value.is_finite() {
        return value;
    }
    let mut i = value.to_bits();
    if value > 0.0 {
        i -= 1;
    } else if value < -0.0 {
        i += 1;
    } else {
        i = 0x8000_0001;
    }
    f32::from_bits(i)
}

/// `signf` — ±1.
#[inline]
pub fn signf(f: f32) -> f32 {
    if f < 0.0 {
        -1.0
    } else {
        1.0
    }
}

/// `compatible_signf` — -1, 0, or 1.
#[inline]
pub fn compatible_signf(f: f32) -> f32 {
    if f > 0.0 {
        1.0
    } else if f < 0.0 {
        -1.0
    } else {
        0.0
    }
}

/// `signum_i_ex`.
#[inline]
pub fn signum_i_ex(a: f32, eps: f32) -> i32 {
    if a > eps {
        1
    } else if a < -eps {
        -1
    } else {
        0
    }
}

/// `signum_i`.
#[inline]
pub fn signum_i(a: f32) -> i32 {
    if a > 0.0 {
        1
    } else if a < 0.0 {
        -1
    } else {
        0
    }
}

/// `integer_digits_f`.
#[inline]
pub fn integer_digits_f(f: f32) -> i32 {
    if f == 0.0 {
        0
    } else {
        f.abs().log10().floor() as i32 + 1
    }
}

/// `integer_digits_d`.
#[inline]
pub fn integer_digits_d(d: f64) -> i32 {
    if d == 0.0 {
        0
    } else {
        d.abs().log10().floor() as i32 + 1
    }
}

/// `integer_digits_i`.
#[inline]
pub fn integer_digits_i(i: i32) -> i32 {
    (i as f64).log10() as i32 + 1
}

/// `unit_float_to_uchar_clamp`.
#[inline]
pub fn unit_float_to_uchar_clamp(val: f32) -> u8 {
    if val <= 0.0 {
        0
    } else if val > 1.0 - 0.5 / 255.0 {
        255
    } else {
        (255.0 * val + 0.5) as u8
    }
}

/// `unit_float_to_ushort_clamp`.
#[inline]
pub fn unit_float_to_ushort_clamp(val: f32) -> u16 {
    if val >= 1.0 - 0.5 / 65535.0 {
        65535
    } else if val <= 0.0 {
        0
    } else {
        (val * 65535.0 + 0.5) as u16
    }
}

/// `unit_ushort_to_uchar`.
#[inline]
pub fn unit_ushort_to_uchar(val: u16) -> u8 {
    if val >= 65535 - 128 {
        255
    } else {
        ((val + 128) >> 8) as u8
    }
}
