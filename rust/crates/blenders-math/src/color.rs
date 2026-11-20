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

/* ---- math_color.cc ---- */

/// `hsv_to_rgb` — h,s,v in [0,1] → rgb.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let nr = ((h * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let ng = (2.0 - (h * 6.0 - 2.0).abs()).clamp(0.0, 1.0);
    let nb = (2.0 - (h * 6.0 - 4.0).abs()).clamp(0.0, 1.0);
    [
        ((nr - 1.0) * s + 1.0) * v,
        ((ng - 1.0) * s + 1.0) * v,
        ((nb - 1.0) * s + 1.0) * v,
    ]
}

/// `hsl_to_rgb`.
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> [f32; 3] {
    let nr = ((h * 6.0 - 3.0).abs() - 1.0).clamp(0.0, 1.0);
    let ng = (2.0 - (h * 6.0 - 2.0).abs()).clamp(0.0, 1.0);
    let nb = (2.0 - (h * 6.0 - 4.0).abs()).clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * l - 1.0).abs()) * s;
    [
        (nr - 0.5) * chroma + l,
        (ng - 0.5) * chroma + l,
        (nb - 0.5) * chroma + l,
    ]
}

/// `BLI_YUV_*` colorspaces.
pub const BLI_YUV_ITU_BT601: i32 = 0;
pub const BLI_YUV_ITU_BT709: i32 = 1;

/// `rgb_to_yuv` — returns (y, u, v).
pub fn rgb_to_yuv(r: f32, g: f32, b: f32, colorspace: i32) -> (f32, f32, f32) {
    match colorspace {
        BLI_YUV_ITU_BT601 => (
            0.299 * r + 0.587 * g + 0.114 * b,
            -0.147 * r - 0.289 * g + 0.436 * b,
            0.615 * r - 0.515 * g - 0.100 * b,
        ),
        _ => (
            0.2126 * r + 0.7152 * g + 0.0722 * b,
            -0.09991 * r - 0.33609 * g + 0.436 * b,
            0.615 * r - 0.55861 * g - 0.05639 * b,
        ),
    }
}

/// `yuv_to_rgb` — returns (r, g, b).
pub fn yuv_to_rgb(y: f32, u: f32, v: f32, colorspace: i32) -> (f32, f32, f32) {
    match colorspace {
        BLI_YUV_ITU_BT601 => (
            y + 1.140 * v,
            y - 0.394 * u - 0.581 * v,
            y + 2.032 * u,
        ),
        _ => (
            y + 1.28033 * v,
            y - 0.21482 * u - 0.38059 * v,
            y + 2.12798 * u,
        ),
    }
}

/// `BLI_YCC_*` colorspaces.
pub const BLI_YCC_ITU_BT601: i32 = 0;
pub const BLI_YCC_ITU_BT709: i32 = 1;
pub const BLI_YCC_JFIF_0_255: i32 = 2;

/// `rgb_to_ycc` — (y, cb, cr) in 0-255 range.
pub fn rgb_to_ycc(r: f32, g: f32, b: f32, colorspace: i32) -> (f32, f32, f32) {
    let (sr, sg, sb) = (255.0 * r, 255.0 * g, 255.0 * b);
    match colorspace {
        BLI_YCC_ITU_BT601 => (
            (0.257 * sr) + (0.504 * sg) + (0.098 * sb) + 16.0,
            (-0.148 * sr) - (0.291 * sg) + (0.439 * sb) + 128.0,
            (0.439 * sr) - (0.368 * sg) - (0.071 * sb) + 128.0,
        ),
        BLI_YCC_ITU_BT709 => (
            (0.183 * sr) + (0.614 * sg) + (0.062 * sb) + 16.0,
            (-0.101 * sr) - (0.338 * sg) + (0.439 * sb) + 128.0,
            (0.439 * sr) - (0.399 * sg) - (0.040 * sb) + 128.0,
        ),
        _ => (
            (0.299 * sr) + (0.587 * sg) + (0.114 * sb),
            (-0.16874 * sr) - (0.33126 * sg) + (0.5 * sb) + 128.0,
            (0.5 * sr) - (0.41869 * sg) - (0.08131 * sb) + 128.0,
        ),
    }
}

/// `ycc_to_rgb` — (r, g, b) in 0-1 range.
pub fn ycc_to_rgb(y: f32, cb: f32, cr: f32, colorspace: i32) -> (f32, f32, f32) {
    let (r, g, b) = match colorspace {
        BLI_YCC_ITU_BT601 => (
            1.164 * (y - 16.0) + 1.596 * (cr - 128.0),
            1.164 * (y - 16.0) - 0.813 * (cr - 128.0) - 0.392 * (cb - 128.0),
            1.164 * (y - 16.0) + 2.017 * (cb - 128.0),
        ),
        BLI_YCC_ITU_BT709 => (
            1.164 * (y - 16.0) + 1.793 * (cr - 128.0),
            1.164 * (y - 16.0) - 0.534 * (cr - 128.0) - 0.213 * (cb - 128.0),
            1.164 * (y - 16.0) + 2.115 * (cb - 128.0),
        ),
        _ => (
            y + 1.402 * cr - 179.456,
            y - 0.34414 * cb - 0.71414 * cr + 135.45984,
            y + 1.772 * cb - 226.816,
        ),
    };
    (r / 255.0, g / 255.0, b / 255.0)
}

fn parse_hex_nibbles(s: &str, widths: &[usize]) -> Option<Vec<u32>> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(widths.len());
    let mut i = 0;
    for &w in widths {
        if i + w > bytes.len() {
            return None;
        }
        let part = std::str::from_utf8(&bytes[i..i + w]).ok()?;
        out.push(u32::from_str_radix(part, 16).ok()?);
        i += w;
    }
    Some(out)
}

/// `hex_to_rgba` — parse "#RRGGBB"/"#RGB"/etc → ([r,g,b], Option<alpha>).
pub fn hex_to_rgba(hexcol: &str) -> Option<([f32; 3], Option<f32>)> {
    let hexcol = hexcol.strip_prefix('#').unwrap_or(hexcol);
    let len = hexcol.len();
    let (vals, alpha_parsed) = match len {
        8 => (parse_hex_nibbles(hexcol, &[2, 2, 2, 2])?, true),
        7 => {
            let v = parse_hex_nibbles(hexcol, &[2, 2, 2, 1])?;
            (vec![v[0], v[1], v[2], v[3] << 4], true)
        }
        6 => (parse_hex_nibbles(hexcol, &[2, 2, 2])?, false),
        5 => {
            let v = parse_hex_nibbles(hexcol, &[1, 1, 1, 2])?;
            (vec![
                (v[0] << 4) | v[0],
                (v[1] << 4) | v[1],
                (v[2] << 4) | v[2],
                v[3],
            ], true)
        }
        4 => {
            let v = parse_hex_nibbles(hexcol, &[1, 1, 1, 1])?;
            (vec![
                (v[0] << 4) | v[0],
                (v[1] << 4) | v[1],
                (v[2] << 4) | v[2],
                (v[3] << 4) | v[3],
            ], true)
        }
        3 => {
            let v = parse_hex_nibbles(hexcol, &[1, 1, 1])?;
            (vec![
                (v[0] << 4) | v[0],
                (v[1] << 4) | v[1],
                (v[2] << 4) | v[2],
            ], false)
        }
        2 => {
            let v = parse_hex_nibbles(hexcol, &[2])?;
            (vec![v[0], v[0], v[0]], false)
        }
        1 => {
            let v = parse_hex_nibbles(hexcol, &[1])?;
            let r = (v[0] << 4) | v[0];
            (vec![r, r, r], false)
        }
        _ => return None,
    };
    let scale = 1.0 / 255.0;
    let rgb = [
        (vals[0] as f32 * scale).clamp(0.0, 1.0),
        (vals[1] as f32 * scale).clamp(0.0, 1.0),
        (vals[2] as f32 * scale).clamp(0.0, 1.0),
    ];
    let a = if alpha_parsed {
        Some((vals[3] as f32 * scale).clamp(0.0, 1.0))
    } else {
        None
    };
    Some((rgb, a))
}

/// `hex_to_rgb`.
pub fn hex_to_rgb(hexcol: &str) -> Option<[f32; 3]> {
    hex_to_rgba(hexcol).map(|(rgb, _)| rgb)
}

/// `rgb_to_hsv` — (h, s, v).
pub fn rgb_to_hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let (mut r, mut g, mut b) = (r, g, b);
    let mut k = 0.0f32;
    if g < b {
        std::mem::swap(&mut g, &mut b);
        k = -1.0;
    }
    let mut min_gb = b;
    if r < g {
        std::mem::swap(&mut r, &mut g);
        k = -2.0 / 6.0 - k;
        min_gb = g.min(b);
    }
    let chroma = r - min_gb;
    (
        (k + (g - b) / (6.0 * chroma + 1e-20)).abs(),
        chroma / (r + 1e-20),
        r,
    )
}

/// `rgb_to_hsl` — (h, s, l).
pub fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let l = (1.0f32).min((cmax + cmin) / 2.0);
    let (mut h, mut s) = (0.0f32, 0.0f32);
    if cmax != cmin {
        let d = cmax - cmin;
        s = if l > 0.5 {
            d / (2.0 - cmax - cmin)
        } else {
            d / (cmax + cmin)
        };
        h = if cmax == r {
            (g - b) / d + if g < b { 6.0 } else { 0.0 }
        } else if cmax == g {
            (b - r) / d + 2.0
        } else {
            (r - g) / d + 4.0
        };
    }
    (h / 6.0, s, l)
}

/// `rgb_to_hsl_compat` — stays close to `orig` (h,s,l) on edge cases.
pub fn rgb_to_hsl_compat(r: f32, g: f32, b: f32, orig: (f32, f32, f32)) -> (f32, f32, f32) {
    const THRESHOLD: f32 = 1e-5;
    let (orig_h, orig_s, _) = orig;
    let (mut h, mut s, l) = rgb_to_hsl(r, g, b);
    if l <= THRESHOLD || s <= THRESHOLD {
        h = orig_h;
        if l <= THRESHOLD {
            s = orig_s;
        }
    }
    if h.abs() <= THRESHOLD && (orig_h - 1.0).abs() <= THRESHOLD {
        h = 1.0;
    } else if (h - 1.0).abs() <= THRESHOLD && orig_h.abs() <= THRESHOLD {
        h = 0.0;
    }
    (h, s, l)
}

/// `rgb_to_hsv_compat`.
pub fn rgb_to_hsv_compat(r: f32, g: f32, b: f32, orig: (f32, f32, f32)) -> (f32, f32, f32) {
    const THRESHOLD: f32 = 1e-5;
    let (orig_h, orig_s, _) = orig;
    let (mut h, mut s, v) = rgb_to_hsv(r, g, b);
    if v <= THRESHOLD {
        h = orig_h;
        s = orig_s;
    } else if s <= THRESHOLD {
        h = orig_h;
    }
    if h.abs() <= THRESHOLD && (orig_h - 1.0).abs() <= THRESHOLD {
        h = 1.0;
    } else if (h - 1.0).abs() <= THRESHOLD && orig_h.abs() <= THRESHOLD {
        h = 0.0;
    }
    (h, s, v)
}

/// `hsv_clamp_v`.
pub fn hsv_clamp_v(hsv: &mut [f32; 3], v_max: f32) {
    if hsv[0] < 0.0 || hsv[0] > 1.0 {
        hsv[0] -= hsv[0].floor();
    }
    hsv[1] = hsv[1].clamp(0.0, 1.0);
    hsv[2] = hsv[2].clamp(0.0, v_max);
}

/// `hsv_to_cpack`.
pub fn hsv_to_cpack(h: f32, s: f32, v: f32) -> u32 {
    let rgb = hsv_to_rgb(h, s, v);
    let (r, g, b) = (
        (rgb[0] * 255.0) as u32,
        (rgb[1] * 255.0) as u32,
        (rgb[2] * 255.0) as u32,
    );
    r + g * 256 + b * 256 * 256
}

/// `rgb_to_cpack`.
pub fn rgb_to_cpack(r: f32, g: f32, b: f32) -> u32 {
    let ir = (255.0 * r.max(0.0)).floor() as u32;
    let ig = (255.0 * g.max(0.0)).floor() as u32;
    let ib = (255.0 * b.max(0.0)).floor() as u32;
    ir.min(255) + ig.min(255) * 256 + ib.min(255) * 256 * 256
}

/// `cpack_to_rgb`.
pub fn cpack_to_rgb(col: u32) -> [f32; 3] {
    [
        (col & 0xFF) as f32 * (1.0 / 255.0),
        ((col >> 8) & 0xFF) as f32 * (1.0 / 255.0),
        ((col >> 16) & 0xFF) as f32 * (1.0 / 255.0),
    ]
}

/// `linearrgb_to_srgb_uchar4_n` — batch conversion with optional 3x3 matrix.
pub fn linearrgb_to_srgb_uchar4_n(
    linear: &[[f32; 4]],
    matrix: Option<&crate::matrix::Mat3>,
) -> Vec<[u8; 4]> {
    linear
        .iter()
        .map(|px| {
            let xyz = crate::vector::Vec3::new(px[0], px[1], px[2]);
            let s3 = if let Some(m) = matrix {
                let t = *m * xyz;
                [t.x, t.y, t.z]
            } else {
                [xyz.x, xyz.y, xyz.z]
            };
            let s = linearrgb_to_srgb_v3(s3);
            [
                unit_float_to_uchar_clamp(s[0]),
                unit_float_to_uchar_clamp(s[1]),
                unit_float_to_uchar_clamp(s[2]),
                unit_float_to_uchar_clamp(px[3]),
            ]
        })
        .collect()
}

/// `rgb_float_set_hue_float_offset`.
pub fn rgb_float_set_hue_float_offset(rgb: &mut [f32; 3], hue_offset: f32) {
    let (mut h, s, v) = rgb_to_hsv(rgb[0], rgb[1], rgb[2]);
    h += hue_offset;
    if h > 1.0 {
        h -= 1.0;
    } else if h < 0.0 {
        h += 1.0;
    }
    *rgb = hsv_to_rgb(h, s, v);
}

/// `rgb_byte_set_hue_float_offset`.
pub fn rgb_byte_set_hue_float_offset(rgb: &mut [u8; 3], hue_offset: f32) {
    let mut f = rgb_uchar_to_float(*rgb);
    rgb_float_set_hue_float_offset(&mut f, hue_offset);
    *rgb = rgb_float_to_uchar(f);
}

/* Whitepoint / Planckian locus (math:: namespace). */

struct LocusEntry {
    mired: f32,
    uv: [f32; 2],
    t: f32,
}
impl LocusEntry {
    fn dist(&self, p: [f32; 2]) -> f32 {
        let diff = [p[0] - self.uv[0], p[1] - self.uv[1]];
        diff[1] - self.t * diff[0]
    }
}

#[rustfmt::skip]
const PLANCK_LOCUS: [LocusEntry; 31] = [
    LocusEntry { mired: 0.0,   uv: [0.18006, 0.26352], t: -0.24341 },
    LocusEntry { mired: 10.0,  uv: [0.18066, 0.26589], t: -0.25479 },
    LocusEntry { mired: 20.0,  uv: [0.18133, 0.26846], t: -0.26876 },
    LocusEntry { mired: 30.0,  uv: [0.18208, 0.27119], t: -0.28539 },
    LocusEntry { mired: 40.0,  uv: [0.18293, 0.27407], t: -0.30470 },
    LocusEntry { mired: 50.0,  uv: [0.18388, 0.27709], t: -0.32675 },
    LocusEntry { mired: 60.0,  uv: [0.18494, 0.28021], t: -0.35156 },
    LocusEntry { mired: 70.0,  uv: [0.18611, 0.28342], t: -0.37915 },
    LocusEntry { mired: 80.0,  uv: [0.18740, 0.28668], t: -0.40955 },
    LocusEntry { mired: 90.0,  uv: [0.18880, 0.28997], t: -0.44278 },
    LocusEntry { mired: 100.0, uv: [0.19032, 0.29326], t: -0.47888 },
    LocusEntry { mired: 125.0, uv: [0.19462, 0.30141], t: -0.58204 },
    LocusEntry { mired: 150.0, uv: [0.19962, 0.30921], t: -0.70471 },
    LocusEntry { mired: 175.0, uv: [0.20525, 0.31647], t: -0.84901 },
    LocusEntry { mired: 200.0, uv: [0.21142, 0.32312], t: -1.0182 },
    LocusEntry { mired: 225.0, uv: [0.21807, 0.32909], t: -1.2168 },
    LocusEntry { mired: 250.0, uv: [0.22511, 0.33439], t: -1.4512 },
    LocusEntry { mired: 275.0, uv: [0.23247, 0.33904], t: -1.7298 },
    LocusEntry { mired: 300.0, uv: [0.24010, 0.34308], t: -2.0637 },
    LocusEntry { mired: 325.0, uv: [0.24792, 0.34655], t: -2.4681 },
    LocusEntry { mired: 350.0, uv: [0.25591, 0.34951], t: -2.9641 },
    LocusEntry { mired: 375.0, uv: [0.26400, 0.35200], t: -3.5814 },
    LocusEntry { mired: 400.0, uv: [0.27218, 0.35407], t: -4.3633 },
    LocusEntry { mired: 425.0, uv: [0.28039, 0.35577], t: -5.3762 },
    LocusEntry { mired: 450.0, uv: [0.28863, 0.35714], t: -6.7262 },
    LocusEntry { mired: 475.0, uv: [0.29685, 0.35823], t: -8.5955 },
    LocusEntry { mired: 500.0, uv: [0.30505, 0.35907], t: -11.324 },
    LocusEntry { mired: 525.0, uv: [0.31320, 0.35968], t: -15.628 },
    LocusEntry { mired: 550.0, uv: [0.32129, 0.36011], t: -23.325 },
    LocusEntry { mired: 575.0, uv: [0.32931, 0.36038], t: -40.770 },
    LocusEntry { mired: 600.0, uv: [0.33724, 0.36051], t: -116.45 },
];

fn interp2(a: [f32; 2], b: [f32; 2], f: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]
}
fn len2(v: [f32; 2]) -> f32 {
    (v[0] * v[0] + v[1] * v[1]).sqrt()
}
fn norm2(v: [f32; 2]) -> [f32; 2] {
    let l = len2(v);
    if l == 0.0 {
        [0.0; 2]
    } else {
        [v[0] / l, v[1] / l]
    }
}

/// `whitepoint_to_temp_tint` — XYZ white → (temperature, tint) or None.
pub fn whitepoint_to_temp_tint(white: [f32; 3]) -> Option<(f32, f32)> {
    let d = white[0] + 15.0 * white[1] + 3.0 * white[2];
    let uv = [4.0 * white[0] / d, 6.0 * white[1] / d];
    // first entry whose dist(uv) >= 0 (upper_bound on dist<0)
    let i = PLANCK_LOCUS.partition_point(|e| e.dist(uv) < 0.0);
    if i == 0 || i == PLANCK_LOCUS.len() {
        return None;
    }
    let (low, high) = (&PLANCK_LOCUS[i - 1], &PLANCK_LOCUS[i]);
    let d_low = low.dist(uv) / (1.0 + low.t * low.t).sqrt();
    let d_high = high.dist(uv) / (1.0 + high.t * high.t).sqrt();
    let f = d_low / (d_low - d_high);
    let uv_temp = interp2(low.uv, high.uv, f);
    let abs_tint = len2([uv[0] - uv_temp[0], uv[1] - uv_temp[1]]) * 3000.0;
    if abs_tint > 150.0 {
        return None;
    }
    let temperature = 1e6 / (low.mired + (high.mired - low.mired) * f);
    let tint = abs_tint * if uv[0] < uv_temp[0] { 1.0 } else { -1.0 };
    Some((temperature, tint))
}

/// `whitepoint_from_temp_tint` — (temperature, tint) → XYZ (Y=1).
pub fn whitepoint_from_temp_tint(temperature: f32, tint: f32) -> [f32; 3] {
    let mired = (1e6 / temperature).clamp(
        PLANCK_LOCUS[0].mired,
        PLANCK_LOCUS[PLANCK_LOCUS.len() - 1].mired,
    );
    let i = PLANCK_LOCUS.partition_point(|e| e.mired < mired);
    let (low, high) = (&PLANCK_LOCUS[i - 1], &PLANCK_LOCUS[i]);
    let f = (mired - low.mired) / (high.mired - low.mired);
    let mut uv = interp2(low.uv, high.uv, f);
    let isotherm0 = norm2([1.0, low.t]);
    let isotherm1 = norm2([1.0, high.t]);
    let isotherm = norm2(interp2(isotherm0, isotherm1, f));
    uv[0] -= isotherm[0] * tint / 3000.0;
    uv[1] -= isotherm[1] * tint / 3000.0;
    let x = 3.0 * uv[0] / (2.0 * uv[0] - 8.0 * uv[1] + 4.0);
    let y = 2.0 * uv[1] / (2.0 * uv[0] - 8.0 * uv[1] + 4.0);
    [x / y, 1.0, (1.0 - x - y) / y]
}

/// `chromatic_adaption_matrix` — Bradford XYZ→LMS adaptation.
pub fn chromatic_adaption_matrix(from_xyz: [f32; 3], to_xyz: [f32; 3]) -> crate::matrix::Mat3 {
    use crate::matrix::Mat3;
    use crate::vector::Vec3;
    // Bradford rows (row-major source); from_cols transposes.
    let bradford = Mat3::from_cols(
        Vec3::new(0.8951, -0.7502, 0.0389),
        Vec3::new(0.2664, 1.7135, -0.0685),
        Vec3::new(-0.1614, 0.0367, 1.0296),
    );
    let from = Vec3::from(from_xyz);
    let to = Vec3::from(to_xyz);
    let from_lms = (bradford * from) / from.y;
    let to_lms = (bradford * to) / to.y;
    let scale = Mat3::from_scale(Vec3::new(
        to_lms.x / from_lms.x,
        to_lms.y / from_lms.y,
        to_lms.z / from_lms.z,
    ));
    bradford.inverted().unwrap_or(Mat3::IDENTITY) * scale * bradford
}
