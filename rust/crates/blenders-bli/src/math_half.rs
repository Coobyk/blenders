//! Port of `math_half.cc` — IEEE-754 binary16 conversions
//! (`math::float_to_half`, `float_to_half_make_finite`, `half_to_float`
//! and the `_array` variants). Scalar version of the NEON/F16C/SSE2 paths.

/// `math::float_to_half` — Fabian Giesen's bit-twiddling conversion.
pub fn float_to_half(v: f32) -> u16 {
    let mut f = v.to_bits();
    let f32infty: u32 = 255 << 23;
    let f16max: u32 = (127 + 16) << 23;
    let denorm_magic: u32 = ((127 - 15) + (23 - 10) + 1) << 23;
    let sign_mask: u32 = 0x8000_0000;
    let mut o: u16;
    let sign = f & sign_mask;
    f ^= sign;
    if f >= f16max {
        o = if f > f32infty { 0x7e00 } else { 0x7c00 };
    } else if f < (113 << 23) {
        let fv = f32::from_bits(f) + f32::from_bits(denorm_magic);
        o = (fv.to_bits() - denorm_magic) as u16;
    } else {
        let mant_odd = (f >> 13) & 1;
        f = f
            .wrapping_add(((15u32.wrapping_sub(127)) << 23).wrapping_add(0xfff))
            .wrapping_add(mant_odd);
        o = (f >> 13) as u16;
    }
    o | (sign >> 16) as u16
}

/// `math::float_to_half_make_finite` — Inf→max, NaN→0.
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

/// `math::half_to_float`.
pub fn half_to_float(v: u16) -> f32 {
    let magic: u32 = 113 << 23;
    let shifted_exp: u32 = 0x7c00 << 13;
    let mut ou = ((v as u32) & 0x7fff) << 13;
    let exp = shifted_exp & ou;
    ou = ou.wrapping_add((127 - 15) << 23);
    if exp == shifted_exp {
        ou = ou.wrapping_add((128 - 16) << 23);
    } else if exp == 0 {
        ou = ou.wrapping_add(1 << 23);
        let of = f32::from_bits(ou) - f32::from_bits(magic);
        return f32::from_bits(of.to_bits() | (((v as u32) & 0x8000) << 16));
    }
    f32::from_bits(ou | (((v as u32) & 0x8000) << 16))
}

/// `math::float_to_half_array`.
pub fn float_to_half_array(src: &[f32], dst: &mut [u16]) {
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        *d = float_to_half(*s);
    }
}

/// `math::float_to_half_clamp_array` — NaN→0, then clamp to `[min,max]`.
pub fn float_to_half_clamp_array(src: &[f32], dst: &mut [u16], min_value: f32, max_value: f32) {
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        let mut v = s.clamp(min_value, max_value);
        if v.is_nan() {
            v = 0.0;
        }
        *d = float_to_half(v);
    }
}

/// `math::float_to_half_make_finite_array`.
pub fn float_to_half_make_finite_array(src: &[f32], dst: &mut [u16]) {
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        *d = float_to_half_make_finite(*s);
    }
}

/// `math::half_to_float_array`.
pub fn half_to_float_array(src: &[u16], dst: &mut [f32]) {
    for (s, d) in src.iter().zip(dst.iter_mut()) {
        *d = half_to_float(*s);
    }
}
