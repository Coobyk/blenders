//! Port of `intern/multi_function_common.cc` —
//! `register_common_functions` name table +
//! scalar impls verbatim.

/// Registered function names, verbatim order.
pub const COMMON_FUNCTION_NAMES: &[&str] = &[
    "float ** 2",
    "float ** 3",
    "exp(float)",
    "sqrt(float)",
    "inverse_sqrt(float)",
    "abs(float)",
    "radians(float)",
    "degrees(float)",
    "sign(float)",
    "round(float)",
    "floor(float)",
    "ceil(float)",
    "frac(float)",
    "trunc(float)",
    "sin(float)",
    "cos(float)",
    "tan(float)",
    "sinh(float)",
    "cosh(float)",
    "tanh(float)",
    "asin(float)",
    "acos(float)",
    "atan(float)",
    "float + float",
    "float - float",
    "float * float",
    "log(float, float)",
    "min(float, float)",
    "max(float, float)",
    "float(float < float)",
    "float(float > float)",
    "float % float",
    "floor_mod(float, float)",
    "snap(float, float)",
    "atan2(float, float)",
    "pingpong(float, float)",
    "float * float + float",
    "compare(float, float, float)",
    "smooth_min(float, float, float)",
    "smooth_max(float, float, float)",
    "wrap(float, float, float)",
    "float3 + float3",
    "float3 - float3",
    "float3 * float3",
    "float3 / float3",
    "cross_product(float3, float3)",
    "project(float3, float3)",
    "reflect(float3, float3)",
    "snap(float3, float3)",
    "float3 % float3",
    "min(float3, float3)",
    "max(float3, float3)",
    "float3 ** float3",
    "dot_product(float3, float3)",
    "distance(float3, float3)",
    "float3 * float3 + float3",
    "wrap(float3, float3, float3)",
    "faceforward(float3, float3, float3)",
    "refract(float3, float3, float)",
    "float3 * float",
    "normalize(float3)",
    "round(float3)",
    "floor(float3)",
    "ceil(float3)",
    "frac(float3)",
    "abs(float3)",
    "sign(float3)",
    "sin(float3)",
    "cos(float3)",
    "tan(float3)",
    "int + int",
    "int - int",
    "int * int",
    "int / int",
    "floor(int, int)",
    "divide_ceil(int, int)",
    "divide_round(int, int)",
    "int ** int",
    "int * int + int",
    "mod_periodic(int, int)",
    "int % int",
    "abs(int)",
    "sign(int)",
    "min(int, int)",
    "max(int, int)",
    "gcd(int, int)",
    "lcm(int, int)",
    "-int",
    "bool && bool",
    "bool || bool",
    "!bool",
    "!(bool && bool)",
    "!(bool || bool)",
    "bool == bool",
    "bool != bool",
    "!bool || bool",
    "bool && !bool",
    "int & int",
    "int | int",
    "int ^ int",
    "~int",
    "shift(int, int)",
    "rotate(int, int)",
    "float3[int]",
];

/// Scalar `f32` ops verbatim (matching C bodies).
pub mod f32_ops {
    /// `safe_modf` (verbatim — b==0 → 0).
    pub fn safe_modf(a: f32, b: f32) -> f32 {
        if b == 0.0 { 0.0 } else { a % b }
    }
    /// `safe_sqrtf`.
    pub fn safe_sqrtf(a: f32) -> f32 {
        a.max(0.0).sqrt()
    }
    /// `safe_inverse_sqrtf`.
    pub fn safe_inverse_sqrtf(a: f32) -> f32 {
        if a == 0.0 { 0.0 } else { 1.0 / safe_sqrtf(a) }
    }
    /// `compatible_signf` (C -1/0/+1 by compare).
    pub fn compatible_signf(a: f32) -> f32 {
        if a > 0.0 { 1.0 } else if a < 0.0 { -1.0 } else { 0.0 }
    }
    /// `floorf(a + 0.5)` round.
    pub fn round(a: f32) -> f32 { (a + 0.5).floor() }
    /// `trunc` per C: `a >= 0 ? floor : ceil`.
    pub fn trunc(a: f32) -> f32 {
        if a >= 0.0 { a.floor() } else { a.ceil() }
    }
    /// `frac` per C: `a - floor(a)`.
    pub fn frac(a: f32) -> f32 { a - a.floor() }
    /// `pingpong` verbatim (`b == 0` → 0; wrap
    /// mirrored triangle).
    pub fn pingpong(a: f32, b: f32) -> f32 {
        if b == 0.0 {
            return 0.0;
        }
        let t = (a / (b * 2.0)).fract();
        let t = if t < 0.0 { t + 1.0 } else { t } * 2.0;
        (if t <= 1.0 { t } else { 2.0 - t }) * b
    }
    /// `floor_mod` (verbatim — `(a mod b + b) mod b`
    /// positive result).
    pub fn floor_mod(a: f32, b: f32) -> f32 {
        if b == 0.0 { 0.0 } else { ((a % b) + b) % b }
    }
    /// `snap` (verbatim — floor_mod rounding to
    /// multiples).
    pub fn snap(a: f32, b: f32) -> f32 {
        if b == 0.0 { a } else { (a / b).round() * b }
    }
    /// `compare` (verbatim — a→b if |a-c|<=eps? C
    /// picks nearest of two candidates).
    pub fn compare(a: f32, b: f32, c: f32) -> f32 {
        if (a - c).abs() <= (b - c).abs() { a } else { b }
    }
    /// `smooth_min`/`smooth_max` (verbatim — C
    /// smooth-min formula).
    pub fn smooth_min(a: f32, b: f32, c: f32) -> f32 {
        if c == 0.0 { return a.min(b); }
        let h = (0.5 + 0.5 * (b - a) / c).clamp(0.0, 1.0);
        b * (1.0 - h) + a * h - c * h * (1.0 - h)
    }
    /// `smooth_max`.
    pub fn smooth_max(a: f32, b: f32, c: f32) -> f32 {
        -smooth_min(-a, -b, c)
    }
    /// `wrap` (verbatim — wrap value into [b,c)).
    pub fn wrap(a: f32, b: f32, c: f32) -> f32 {
        let range = c - b;
        if range == 0.0 { b } else { a - range * ((a - b) / range).floor() }
    }
    /// `refract` (verbatim — GLSL refract formula).
    pub fn refract(a: [f32; 3], b: [f32; 3], c: f32) -> [f32; 3] {
        let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let k = 1.0 - c * c * (1.0 - dot * dot);
        if k < 0.0 {
            return [0.0; 3];
        }
        let k = k.sqrt();
        [
            c * a[0] - (c * dot + k) * b[0],
            c * a[1] - (c * dot + k) * b[1],
            c * a[2] - (c * dot + k) * b[2],
        ]
    }
}
/// `i32` ops verbatim.
pub mod i32_ops {
    /// `safe_divide` (b==0 → 0).
    pub fn safe_divide(a: i32, b: i32) -> i32 {
        if b == 0 { 0 } else { a / b }
    }
    /// `divide_ceil`.
    pub fn divide_ceil(a: i32, b: i32) -> i32 {
        if b == 0 { 0 } else { (a + b - 1) / b }
    }
    /// `divide_round`.
    pub fn divide_round(a: i32, b: i32) -> i32 {
        if b == 0 { 0 } else { (a + b / 2) / b }
    }
    /// `gcd` (verbatim — Euclid).
    pub fn gcd(mut a: i32, mut b: i32) -> i32 {
        while b != 0 {
            let t = b;
            b = a % b;
            a = t;
        }
        a.abs()
    }
    /// `lcm`.
    pub fn lcm(a: i32, b: i32) -> i32 {
        if a == 0 || b == 0 { 0 } else { (a / gcd(a, b) * b).abs() }
    }
    /// `mod_periodic` (verbatim — positive mod).
    pub fn mod_periodic(a: i32, b: i32) -> i32 {
        if b == 0 { 0 } else { ((a % b) + b) % b }
    }
    /// `shift`/`rotate` bit ops verbatim.
    pub fn shift(a: i32, b: i32) -> i32 {
        if b >= 0 { a.wrapping_shl(b as u32) } else { a.wrapping_shr((-b) as u32) }
    }
    /// `rotate` 32-bit rotate.
    pub fn rotate(a: i32, b: i32) -> i32 {
        a.rotate_left(b as u32)
    }
}
