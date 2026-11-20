//! Port of `math_color.cc` (scalar sRGB) + `math_color_inline.cc`.
//!
//! sRGB ↔ linear RGB, uchar conversions, grayscale, dithering, and
//! premultiplied-alpha transforms. SIMD paths collapse to scalar.

use crate::base::{signf, unit_float_to_uchar_clamp};

/// `srgb_to_linearrgb` — per-channel sRGB → linear.
#[inline]
pub fn srgb_to_linearrgb(c: f32) -> f32 {
    if c < 0.04045 {
        if c < 0.0 {
            0.0
        } else {
            c * (1.0 / 12.92)
        }
    } else {
        ((c + 0.055) * (1.0 / 1.055)).powf(2.4)
    }
}

/// `linearrgb_to_srgb` — per-channel linear → sRGB.
#[inline]
pub fn linearrgb_to_srgb(c: f32) -> f32 {
    if c < 0.0031308 {
        if c < 0.0 {
            0.0
        } else {
            c * 12.92
        }
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

/// `srgb_to_linearrgb_v3_v3`.
pub fn srgb_to_linearrgb_v3(srgb: [f32; 3]) -> [f32; 3] {
    [
        srgb_to_linearrgb(srgb[0]),
        srgb_to_linearrgb(srgb[1]),
        srgb_to_linearrgb(srgb[2]),
    ]
}

/// `linearrgb_to_srgb_v3_v3`.
pub fn linearrgb_to_srgb_v3(linear: [f32; 3]) -> [f32; 3] {
    [
        linearrgb_to_srgb(linear[0]),
        linearrgb_to_srgb(linear[1]),
        linearrgb_to_srgb(linear[2]),
    ]
}

/// `srgb_to_linearrgb_v4`.
pub fn srgb_to_linearrgb_v4(srgb: [f32; 4]) -> [f32; 4] {
    let [r, g, b] = srgb_to_linearrgb_v3([srgb[0], srgb[1], srgb[2]]);
    [r, g, b, srgb[3]]
}

/// `linearrgb_to_srgb_v4`.
pub fn linearrgb_to_srgb_v4(linear: [f32; 4]) -> [f32; 4] {
    let [r, g, b] = linearrgb_to_srgb_v3([linear[0], linear[1], linear[2]]);
    [r, g, b, linear[3]]
}

/// `linearrgb_to_srgb_uchar3`.
pub fn linearrgb_to_srgb_uchar3(linear: [f32; 3]) -> [u8; 3] {
    let s = linearrgb_to_srgb_v3(linear);
    [
        unit_float_to_uchar_clamp(s[0]),
        unit_float_to_uchar_clamp(s[1]),
        unit_float_to_uchar_clamp(s[2]),
    ]
}

/// `linearrgb_to_srgb_uchar4`.
pub fn linearrgb_to_srgb_uchar4(linear: [f32; 4]) -> [u8; 4] {
    let s = linearrgb_to_srgb_v4(linear);
    [
        unit_float_to_uchar_clamp(s[0]),
        unit_float_to_uchar_clamp(s[1]),
        unit_float_to_uchar_clamp(s[2]),
        unit_float_to_uchar_clamp(s[3]),
    ]
}

/// `srgb_to_linearrgb_predivide_v4` — for premultiplied alpha.
pub fn srgb_to_linearrgb_predivide_v4(srgb: [f32; 4]) -> [f32; 4] {
    let (alpha, inv_alpha) = if srgb[3] == 1.0 || srgb[3] == 0.0 {
        (1.0, 1.0)
    } else {
        (srgb[3], 1.0 / srgb[3])
    };
    let mut linear = srgb_to_linearrgb_v3([srgb[0] * inv_alpha, srgb[1] * inv_alpha, srgb[2] * inv_alpha]);
    linear[0] *= alpha;
    linear[1] *= alpha;
    linear[2] *= alpha;
    [linear[0], linear[1], linear[2], srgb[3]]
}

/// `linearrgb_to_srgb_predivide_v4`.
pub fn linearrgb_to_srgb_predivide_v4(linear: [f32; 4]) -> [f32; 4] {
    let (alpha, inv_alpha) = if linear[3] == 1.0 || linear[3] == 0.0 {
        (1.0, 1.0)
    } else {
        (linear[3], 1.0 / linear[3])
    };
    let mut srgb =
        linearrgb_to_srgb_v3([linear[0] * inv_alpha, linear[1] * inv_alpha, linear[2] * inv_alpha]);
    srgb[0] *= alpha;
    srgb[1] *= alpha;
    srgb[2] *= alpha;
    [srgb[0], srgb[1], srgb[2], linear[3]]
}

/// `BLI_color_from_srgb_table` equivalent — srgb→linear for a byte index.
#[inline]
pub fn color_from_srgb_table(b: u8) -> f32 {
    srgb_to_linearrgb(b as f32 * (1.0 / 255.0))
}

/// `srgb_to_linearrgb_uchar4`.
pub fn srgb_to_linearrgb_uchar4(srgb: [u8; 4]) -> [f32; 4] {
    [
        color_from_srgb_table(srgb[0]),
        color_from_srgb_table(srgb[1]),
        color_from_srgb_table(srgb[2]),
        srgb[3] as f32 * (1.0 / 255.0),
    ]
}

/// `srgb_to_linearrgb_uchar4_predivide`.
pub fn srgb_to_linearrgb_uchar4_predivide(srgb: [u8; 4]) -> [f32; 4] {
    if srgb[3] == 255 || srgb[3] == 0 {
        return srgb_to_linearrgb_uchar4(srgb);
    }
    let fsrgb = [
        srgb[0] as f32 / 255.0,
        srgb[1] as f32 / 255.0,
        srgb[2] as f32 / 255.0,
        srgb[3] as f32 / 255.0,
    ];
    srgb_to_linearrgb_predivide_v4(fsrgb)
}

/// `rgb_uchar_to_float`.
pub fn rgb_uchar_to_float(col: [u8; 3]) -> [f32; 3] {
    [
        col[0] as f32 / 255.0,
        col[1] as f32 / 255.0,
        col[2] as f32 / 255.0,
    ]
}

/// `rgba_uchar_to_float`.
pub fn rgba_uchar_to_float(col: [u8; 4]) -> [f32; 4] {
    [
        col[0] as f32 / 255.0,
        col[1] as f32 / 255.0,
        col[2] as f32 / 255.0,
        col[3] as f32 / 255.0,
    ]
}

/// `rgb_float_to_uchar`.
pub fn rgb_float_to_uchar(col: [f32; 3]) -> [u8; 3] {
    [
        unit_float_to_uchar_clamp(col[0]),
        unit_float_to_uchar_clamp(col[1]),
        unit_float_to_uchar_clamp(col[2]),
    ]
}

/// `rgba_float_to_uchar`.
pub fn rgba_float_to_uchar(col: [f32; 4]) -> [u8; 4] {
    [
        unit_float_to_uchar_clamp(col[0]),
        unit_float_to_uchar_clamp(col[1]),
        unit_float_to_uchar_clamp(col[2]),
        unit_float_to_uchar_clamp(col[3]),
    ]
}

/// `rgba_uchar_args_test_set` — set only if alpha is 0.
pub fn rgba_uchar_args_test_set(col: &mut [u8; 4], r: u8, g: u8, b: u8, a: u8) {
    if col[3] == 0 {
        *col = [r, g, b, a];
    }
}

/// `cpack_cpy_3ub`.
pub fn cpack_cpy_3ub(pack: u32) -> [u8; 3] {
    [(pack & 0xFF) as u8, ((pack >> 8) & 0xFF) as u8, ((pack >> 16) & 0xFF) as u8]
}

/// `srgb_to_grayscale` — Rec. 709 luma for sRGB UI colors.
pub fn srgb_to_grayscale(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// `srgb_to_grayscale_byte`.
pub fn srgb_to_grayscale_byte(rgb: [u8; 3]) -> u8 {
    ((54 * rgb[0] as u32 + 182 * rgb[1] as u32 + 19 * rgb[2] as u32) / 255) as u8
}

/// `compare_rgb_uchar`.
pub fn compare_rgb_uchar(col_a: [u8; 3], col_b: [u8; 3], limit: i32) -> bool {
    let r = col_a[0] as i32 - col_b[0] as i32;
    if r.abs() < limit {
        let g = col_a[1] as i32 - col_b[1] as i32;
        if g.abs() < limit {
            let b = col_a[2] as i32 - col_b[2] as i32;
            if b.abs() < limit {
                return true;
            }
        }
    }
    false
}

/// `hash_iqint3_f` — 2D hash for dithering (JCGT Vol. 9, No. 3, 2020).
pub fn hash_iqint3_f(x: u32, y: u32) -> f32 {
    let qx = 1103515245u32.wrapping_mul((x >> 1) ^ y);
    let qy = 1103515245u32.wrapping_mul((y >> 1) ^ x);
    let n = 1103515245u32.wrapping_mul(qx ^ (qy >> 3));
    n as f32 * (1.0 / u32::MAX as f32)
}

/// `dither_random_value` — triangle-distributed dither.
pub fn dither_random_value(x: i32, y: i32) -> f32 {
    let mut v = hash_iqint3_f(x as u32, y as u32) * 2.0 - 1.0;
    v = signf(v) * (1.0 - (1.0 - v.abs()).sqrt());
    v
}

/// `float_to_byte_dither_v3`.
pub fn float_to_byte_dither_v3(f: [f32; 3], dither: f32, x: i32, y: i32) -> [u8; 3] {
    let dither_value = dither_random_value(x, y) * 0.0033 * dither;
    [
        unit_float_to_uchar_clamp(dither_value + f[0]),
        unit_float_to_uchar_clamp(dither_value + f[1]),
        unit_float_to_uchar_clamp(dither_value + f[2]),
    ]
}

/// `premul_to_straight_v4_v4`.
pub fn premul_to_straight_v4(premul: [f32; 4]) -> [f32; 4] {
    if premul[3] == 0.0 || premul[3] == 1.0 {
        premul
    } else {
        let alpha_inv = 1.0 / premul[3];
        [
            premul[0] * alpha_inv,
            premul[1] * alpha_inv,
            premul[2] * alpha_inv,
            premul[3],
        ]
    }
}

/// `straight_to_premul_v4_v4`.
pub fn straight_to_premul_v4(straight: [f32; 4]) -> [f32; 4] {
    let alpha = straight[3];
    [
        straight[0] * alpha,
        straight[1] * alpha,
        straight[2] * alpha,
        straight[3],
    ]
}

/// `straight_uchar_to_premul_float`.
pub fn straight_uchar_to_premul_float(color: [u8; 4]) -> [f32; 4] {
    let alpha = color[3] as f32 * (1.0 / 255.0);
    let fac = alpha * (1.0 / 255.0);
    [
        color[0] as f32 * fac,
        color[1] as f32 * fac,
        color[2] as f32 * fac,
        alpha,
    ]
}

/// `premul_float_to_straight_uchar`.
pub fn premul_float_to_straight_uchar(color: [f32; 4]) -> [u8; 4] {
    if color[3] == 0.0 || color[3] == 1.0 {
        rgba_float_to_uchar(color)
    } else {
        let alpha_inv = 1.0 / color[3];
        [
            unit_float_to_uchar_clamp(color[0] * alpha_inv),
            unit_float_to_uchar_clamp(color[1] * alpha_inv),
            unit_float_to_uchar_clamp(color[2] * alpha_inv),
            unit_float_to_uchar_clamp(color[3]),
        ]
    }
}
