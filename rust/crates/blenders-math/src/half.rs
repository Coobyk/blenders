//! Port of `math_half.cc` — IEEE 754 half-float (binary16) conversion.
//!
//! The SIMD paths (F16C/NEON/SSE2) collapse to the scalar round-to-nearest-even
//! algorithms from Fabian "ryg" Giesen's public-domain code, which Blender uses
//! for its non-hardware paths.

/// `math::float_to_half` — f32 → u16 binary16 bits, round-to-nearest-even.
pub fn float_to_half(v: f32) -> u16 {
    let f16max: u32 = (127 + 16) << 23;
    let f32infty: u32 = 255 << 23;
    let denorm_magic_u: u32 = ((127 - 15) + (23 - 10) + 1) << 23;
    let denorm_magic_f = f32::from_bits(denorm_magic_u);
    let sign_mask = 0x8000_0000u32;

    let sign = v.to_bits() & sign_mask;
    let mut fu = v.to_bits() ^ sign;

    let o: u32 = if fu >= f16max {
        // Inf or NaN
        if fu > f32infty { 0x7e00 } else { 0x7c00 }
    } else if fu < (113 << 23) {
        // subnormal or zero: magic-value mantissa alignment
        let f = f32::from_bits(fu) + denorm_magic_f;
        f.to_bits() - denorm_magic_u
    } else {
        let mant_odd = (fu >> 13) & 1;
        fu += ((15 - 127) as u32) << 23 | 0xfff;
        fu += mant_odd;
        fu >> 13
    };
    (o | (sign >> 16)) as u16
}

/// `math::float_to_half_make_finite` — NaN→0, Inf→±max.
pub fn float_to_half_make_finite(v: f32) -> u16 {
    let mut h = float_to_half(v);
    if (h & 0x7c00) == 0x7c00 {
        if (h & 0x03ff) == 0 {
            h ^= 0x07ff;
        } else {
            h &= 0x8000;
        }
    }
    h
}

/// `math::half_to_float` — binary16 bits → f32.
pub fn half_to_float(v: u16) -> f32 {
    let magic: u32 = 113 << 23;
    let shifted_exp: u32 = 0x7c00 << 13;

    let mut ou = ((v & 0x7fff) as u32) << 13;
    let exp = shifted_exp & ou;
    ou = ou.wrapping_add((127 - 15) << 23);

    if exp == shifted_exp {
        ou = ou.wrapping_add((128 - 16) << 23);
    } else if exp == 0 {
        ou = ou.wrapping_add(1 << 23);
        ou = (f32::from_bits(ou) - f32::from_bits(magic)).to_bits();
    }
    ou |= ((v & 0x8000) as u32) << 16;
    f32::from_bits(ou)
}

/// `math::float_to_half_array`.
pub fn float_to_half_array(src: &[f32], dst: &mut [u16]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = float_to_half(*s);
    }
}

/// `math::float_to_half_clamp_array` — NaN→0, clamp to [min,max].
pub fn float_to_half_clamp_array(src: &[f32], dst: &mut [u16], min_value: f32, max_value: f32) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        let v = if s.is_nan() {
            0.0
        } else {
            s.clamp(min_value, max_value)
        };
        *d = float_to_half(v);
    }
}

/// `math::float_to_half_make_finite_array`.
pub fn float_to_half_make_finite_array(src: &[f32], dst: &mut [u16]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = float_to_half_make_finite(*s);
    }
}

/// `math::half_to_float_array`.
pub fn half_to_float_array(src: &[u16], dst: &mut [f32]) {
    for (d, s) in dst.iter_mut().zip(src.iter()) {
        *d = half_to_float(*s);
    }
}
