//! Port of `math_base_safe_inline.cc` — `blender::` safe math helpers.

/// `safe_divide`.
#[inline]
pub fn safe_divide(a: f32, b: f32) -> f32 {
    if b != 0.0 { a / b } else { 0.0 }
}
/// `safe_modf`.
#[inline]
pub fn safe_modf(a: f32, b: f32) -> f32 {
    if b != 0.0 { a % b } else { 0.0 }
}
/// `safe_floored_modf`.
#[inline]
pub fn safe_floored_modf(a: f32, b: f32) -> f32 {
    if b != 0.0 { a - (a / b).floor() * b } else { 0.0 }
}
/// `safe_logf`.
#[inline]
pub fn safe_logf(a: f32, base: f32) -> f32 {
    if a <= 0.0 || base <= 0.0 {
        0.0
    } else {
        safe_divide(a.ln(), base.ln())
    }
}
/// `safe_sqrtf`.
#[inline]
pub fn safe_sqrtf(a: f32) -> f32 {
    a.max(0.0).sqrt()
}
/// `safe_inverse_sqrtf`.
#[inline]
pub fn safe_inverse_sqrtf(a: f32) -> f32 {
    if a > 0.0 { 1.0 / a.sqrt() } else { 0.0 }
}
/// `safe_asinf`.
#[inline]
pub fn safe_asinf(a: f32) -> f32 {
    a.clamp(-1.0, 1.0).asin()
}
/// `safe_acosf`.
#[inline]
pub fn safe_acosf(a: f32) -> f32 {
    a.clamp(-1.0, 1.0).acos()
}
/// `safe_powf` — NaN-free pow for non-integral exponents of negative bases.
#[inline]
pub fn safe_powf(base: f32, exponent: f32) -> f32 {
    if base < 0.0 && exponent != exponent.trunc() {
        0.0
    } else {
        base.powf(exponent)
    }
}
