//! Port of `math_color_blend_inline.cc` — Photoshop-style pixel blend modes.
//! Byte colors are straight alpha; float colors are premultiplied alpha.

use crate::base::{divide_round_i, round_fl_to_uchar_clamp};
use crate::color::{hsv_to_rgb, rgb_to_hsv};

const EPS_SATURATION: f32 = 0.0005;
const EPS_ALPHA: f32 = 0.0005;

macro_rules! noop_or {
    ($dst:expr, $src1:expr) => {
        *$dst = $src1;
    };
}

/* Straight-alpha byte modes (out-params to match C). */

/// `blend_color_mix_byte` — source-over.
pub fn blend_color_mix_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        let mt = 255 - t;
        let tmp3 = (mt * src1[3] as i32) + (t * 255);
        dst[0] = divide_round_i(
            (mt * src1[3] as i32 * src1[0] as i32) + (t * 255 * src2[0] as i32),
            tmp3,
        ) as u8;
        dst[1] = divide_round_i(
            (mt * src1[3] as i32 * src1[1] as i32) + (t * 255 * src2[1] as i32),
            tmp3,
        ) as u8;
        dst[2] = divide_round_i(
            (mt * src1[3] as i32 * src1[2] as i32) + (t * 255 * src2[2] as i32),
            tmp3,
        ) as u8;
        dst[3] = divide_round_i(tmp3, 255) as u8;
    } else {
        noop_or!(dst, src1);
    }
}

macro_rules! byte_mode {
    ($name:ident, $body:expr) => {
        pub fn $name(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
            let fac = src2[3] as i32;
            if fac != 0 {
                let mfac = 255 - fac;
                for i in 0..3 {
                    let s1 = src1[i] as i32;
                    let s2 = src2[i] as i32;
                    let temp: i32 = $body(s1, s2);
                    dst[i] = ((temp * fac + s1 * mfac) / 255) as u8;
                }
            } else {
                *dst = src1;
            }
        }
    };
}

byte_mode!(blend_color_overlay_byte, |s1: i32, s2: i32| {
    if s1 > 127 {
        255 - ((255 - 2 * (s1 - 127)) * (255 - s2) / 255)
    } else {
        (2 * s1 * s2) >> 8
    }
});
byte_mode!(blend_color_hardlight_byte, |s1: i32, s2: i32| {
    if s2 > 127 {
        255 - ((255 - 2 * (s2 - 127)) * (255 - s1) / 255)
    } else {
        (2 * s2 * s1) >> 8
    }
});
byte_mode!(blend_color_burn_byte, |s1: i32, s2: i32| {
    if s2 == 0 {
        0
    } else {
        (255 - ((255 - s1) * 255) / s2).max(0)
    }
});
byte_mode!(blend_color_linearburn_byte, |s1: i32, s2: i32| {
    (s1 + s2 - 255).max(0)
});
byte_mode!(blend_color_dodge_byte, |s1: i32, s2: i32| {
    if s2 == 255 {
        255
    } else {
        ((s1 * 255) / (255 - s2)).min(255)
    }
});
byte_mode!(blend_color_screen_byte, |s1: i32, s2: i32| {
    (255 - (((255 - s1) * (255 - s2)) / 255)).max(0)
});
byte_mode!(blend_color_pinlight_byte, |s1: i32, s2: i32| {
    if s2 > 127 {
        (2 * (s2 - 127)).max(s1)
    } else {
        (2 * s2).min(s1)
    }
});
byte_mode!(blend_color_linearlight_byte, |s1: i32, s2: i32| {
    if s2 > 127 {
        (s1 + 2 * (s2 - 127)).min(255)
    } else {
        (s1 + 2 * s2 - 255).max(0)
    }
});
byte_mode!(blend_color_vividlight_byte, |s1: i32, s2: i32| {
    if s2 == 255 {
        if s1 == 0 {
            127
        } else {
            255
        }
    } else if s2 == 0 {
        if s1 == 255 {
            127
        } else {
            0
        }
    } else if s2 > 127 {
        ((s1 * 255) / (2 * (255 - s2))).min(255)
    } else {
        (255 - (255 - s1) * 255 / (2 * s2)).max(0)
    }
});
byte_mode!(blend_color_difference_byte, |s1: i32, s2: i32| (s1 - s2).abs());
byte_mode!(blend_color_exclusion_byte, |s1: i32, s2: i32| {
    127 - ((2 * (s1 - 127) * (s2 - 127)) / 255).min(127)
});

/// `blend_color_add_byte`.
pub fn blend_color_add_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        for i in 0..3 {
            dst[i] = divide_round_i(src1[i] as i32 * 255 + src2[i] as i32 * t, 255).min(255) as u8;
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_sub_byte`.
pub fn blend_color_sub_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        for i in 0..3 {
            dst[i] = divide_round_i(src1[i] as i32 * 255 - src2[i] as i32 * t, 255).max(0) as u8;
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_mul_byte`.
pub fn blend_color_mul_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        let mt = 255 - t;
        for i in 0..3 {
            let tmp = mt * src1[i] as i32 * 255 + t * src1[i] as i32 * src2[i] as i32;
            dst[i] = divide_round_i(tmp, 255 * 255) as u8;
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_lighten_byte`.
pub fn blend_color_lighten_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        let mt = 255 - t;
        for i in 0..3 {
            let tmp = mt * src1[i] as i32 + t * src1[i].max(src2[i]) as i32;
            dst[i] = divide_round_i(tmp, 255) as u8;
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_darken_byte`.
pub fn blend_color_darken_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        let mt = 255 - t;
        for i in 0..3 {
            let tmp = mt * src1[i] as i32 + t * src1[i].min(src2[i]) as i32;
            dst[i] = divide_round_i(tmp, 255) as u8;
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_erase_alpha_byte`.
pub fn blend_color_erase_alpha_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        dst[0] = src1[0];
        dst[1] = src1[1];
        dst[2] = src1[2];
        dst[3] = (src1[3] as i32 - divide_round_i(t * src2[3] as i32, 255)).max(0) as u8;
    } else {
        *dst = src1;
    }
}

/// `blend_color_add_alpha_byte`.
pub fn blend_color_add_alpha_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    if src2[3] != 0 {
        let t = src2[3] as i32;
        dst[0] = src1[0];
        dst[1] = src1[1];
        dst[2] = src1[2];
        dst[3] = (src1[3] as i32 + divide_round_i(t * src2[3] as i32, 255)).min(255) as u8;
    } else {
        *dst = src1;
    }
}

/// `blend_color_softlight_byte` (Pegtop formula).
pub fn blend_color_softlight_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    let fac = src2[3] as f32 / 255.0;
    if fac != 0.0 {
        let mfac = 1.0 - fac;
        for i in 0..3 {
            let s1 = src1[i] as f32 / 255.0;
            let s2 = src2[i] as f32 / 255.0;
            let screen = 1.0 - (1.0 - s1) * (1.0 - s2);
            let soft_light = ((1.0 - s1) * s2 + screen) * s1;
            dst[i] = round_fl_to_uchar_clamp((s1 * mfac + soft_light * fac) * 255.0);
        }
    } else {
        *dst = src1;
    }
}

fn hsv_byte_common(
    dst: &mut [u8; 4],
    src1: [u8; 4],
    src2: [u8; 4],
    mix: impl Fn((f32, f32, f32), (f32, f32, f32)) -> (f32, f32, f32),
) {
    let fac = src2[3] as i32;
    if fac != 0 {
        let mfac = 255 - fac;
        let hsv1 = rgb_to_hsv(
            src1[0] as f32 / 255.0,
            src1[1] as f32 / 255.0,
            src1[2] as f32 / 255.0,
        );
        let hsv2 = rgb_to_hsv(
            src2[0] as f32 / 255.0,
            src2[1] as f32 / 255.0,
            src2[2] as f32 / 255.0,
        );
        let (h, s, v) = mix(hsv1, hsv2);
        let rgb = hsv_to_rgb(h, s, v);
        for i in 0..3 {
            dst[i] = ((((rgb[i] * 255.0) as i32) * fac + src1[i] as i32 * mfac) / 255) as u8;
        }
    } else {
        *dst = src1;
    }
}

/// `blend_color_color_byte` — take hue+sat from src2, keep src1 value.
pub fn blend_color_color_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    hsv_byte_common(dst, src1, src2, |(_, _, v1), (h2, s2, _)| (h2, s2, v1));
}

/// `blend_color_hue_byte` — take hue from src2.
pub fn blend_color_hue_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    hsv_byte_common(dst, src1, src2, |(_, s1, v1), (h2, _, _)| (h2, s1, v1));
}

/// `blend_color_saturation_byte` — take saturation from src2 (if src1 not achromatic).
pub fn blend_color_saturation_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    hsv_byte_common(dst, src1, src2, |(h1, s1, v1), (_, s2, _)| {
        (h1, if s1 > EPS_SATURATION { s2 } else { s1 }, v1)
    });
}

/// `blend_color_luminosity_byte` — take value from src2.
pub fn blend_color_luminosity_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4]) {
    hsv_byte_common(dst, src1, src2, |(h1, s1, _), (_, _, v2)| (h1, s1, v2));
}

/// `blend_color_interpolate_byte` — premultiplied-space lerp.
pub fn blend_color_interpolate_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4], ft: f32) {
    let t = (255.0 * ft) as i32;
    let mt = 255 - t;
    let tmp = mt * src1[3] as i32 + t * src2[3] as i32;
    if tmp > 0 {
        for i in 0..3 {
            dst[i] = divide_round_i(
                mt * src1[i] as i32 * src1[3] as i32 + t * src2[i] as i32 * src2[3] as i32,
                tmp,
            ) as u8;
        }
        dst[3] = divide_round_i(tmp, 255) as u8;
    } else {
        *dst = src1;
        dst[3] = 0;
    }
}

/* Premultiplied-alpha float modes. */

macro_rules! float_mode {
    ($name:ident, $body:expr) => {
        pub fn $name(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
            let fac = src2[3];
            if fac != 0.0 {
                let mfac = 1.0 - fac;
                for i in 0..3 {
                    let temp: f32 = $body(src1[i], src2[i]);
                    dst[i] = temp * fac + src1[i] * mfac;
                }
            } else {
                *dst = src1;
            }
        }
    };
}

/// `blend_color_mix_float` — premultiplied over.
pub fn blend_color_mix_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        let t = src2[3];
        let mt = 1.0 - t;
        for i in 0..3 {
            dst[i] = mt * src1[i] + src2[i];
        }
        dst[3] = mt * src1[3] + t;
    } else {
        *dst = src1;
    }
}

/// `blend_color_add_float`.
pub fn blend_color_add_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        for i in 0..3 {
            dst[i] = src1[i] + src2[i] * src1[3];
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_sub_float`.
pub fn blend_color_sub_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        for i in 0..3 {
            dst[i] = (src1[i] - src2[i] * src1[3]).max(0.0);
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_mul_float`.
pub fn blend_color_mul_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        let t = src2[3];
        let mt = 1.0 - t;
        for i in 0..3 {
            dst[i] = mt * src1[i] + src1[i] * src2[i] * src1[3];
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_lighten_float`.
pub fn blend_color_lighten_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        let t = src2[3];
        let mt = 1.0 - t;
        let map_alpha = src1[3] / src2[3];
        for i in 0..3 {
            dst[i] = mt * src1[i] + t * src1[i].max(src2[i] * map_alpha);
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_darken_float`.
pub fn blend_color_darken_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 {
        let t = src2[3];
        let mt = 1.0 - t;
        let map_alpha = src1[3] / src2[3];
        for i in 0..3 {
            dst[i] = mt * src1[i] + t * src1[i].min(src2[i] * map_alpha);
        }
        dst[3] = src1[3];
    } else {
        *dst = src1;
    }
}

/// `blend_color_erase_alpha_float`.
pub fn blend_color_erase_alpha_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 && src1[3] > 0.0 {
        let mut alpha = (src1[3] - src2[3]).max(0.0);
        if alpha <= EPS_ALPHA {
            alpha = 0.0;
        }
        let map_alpha = alpha / src1[3];
        dst[0] = src1[0] * map_alpha;
        dst[1] = src1[1] * map_alpha;
        dst[2] = src1[2] * map_alpha;
        dst[3] = alpha;
    } else {
        *dst = src1;
    }
}

/// `blend_color_add_alpha_float`.
pub fn blend_color_add_alpha_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    if src2[3] != 0.0 && src1[3] < 1.0 {
        let mut alpha = (src1[3] + src2[3]).min(1.0);
        if alpha >= 1.0 - EPS_ALPHA {
            alpha = 1.0;
        }
        let map_alpha = if src1[3] > 0.0 { alpha / src1[3] } else { 1.0 };
        dst[0] = src1[0] * map_alpha;
        dst[1] = src1[1] * map_alpha;
        dst[2] = src1[2] * map_alpha;
        dst[3] = alpha;
    } else {
        *dst = src1;
    }
}

float_mode!(blend_color_overlay_float, |s1: f32, s2: f32| {
    if s1 > 0.5 {
        1.0 - (1.0 - 2.0 * (s1 - 0.5)) * (1.0 - s2)
    } else {
        2.0 * s1 * s2
    }
});
float_mode!(blend_color_hardlight_float, |s1: f32, s2: f32| {
    if s2 > 0.5 {
        1.0 - ((1.0 - 2.0 * (s2 - 0.5)) * (1.0 - s1))
    } else {
        2.0 * s2 * s1
    }
});
float_mode!(blend_color_burn_float, |s1: f32, s2: f32| {
    if s2 == 0.0 {
        0.0
    } else {
        (1.0 - (1.0 - s1) / s2).max(0.0)
    }
});
float_mode!(blend_color_linearburn_float, |s1: f32, s2: f32| {
    (s1 + s2 - 1.0).max(0.0)
});
float_mode!(blend_color_dodge_float, |s1: f32, s2: f32| {
    if s2 >= 1.0 {
        1.0
    } else {
        (s1 / (1.0 - s2)).min(1.0)
    }
});
float_mode!(blend_color_screen_float, |s1: f32, s2: f32| {
    (1.0 - ((1.0 - s1) * (1.0 - s2))).max(0.0)
});
float_mode!(blend_color_softlight_float, |s1: f32, s2: f32| {
    let screen = 1.0 - (1.0 - s1) * (1.0 - s2);
    ((1.0 - s1) * s2 + screen) * s1
});
float_mode!(blend_color_pinlight_float, |s1: f32, s2: f32| {
    if s2 > 0.5 {
        (2.0 * (s2 - 0.5)).max(s1)
    } else {
        (2.0 * s2).min(s1)
    }
});
float_mode!(blend_color_linearlight_float, |s1: f32, s2: f32| {
    if s2 > 0.5 {
        (s1 + 2.0 * (s2 - 0.5)).min(1.0)
    } else {
        (s1 + 2.0 * s2 - 1.0).max(0.0)
    }
});
float_mode!(blend_color_vividlight_float, |s1: f32, s2: f32| {
    if s2 == 1.0 {
        if s1 == 0.0 {
            0.5
        } else {
            1.0
        }
    } else if s2 == 0.0 {
        if s1 == 1.0 {
            0.5
        } else {
            0.0
        }
    } else if s2 > 0.5 {
        (s1 / (2.0 * (1.0 - s2))).min(1.0)
    } else {
        (1.0 - (1.0 - s1) / (2.0 * s2)).max(0.0)
    }
});
float_mode!(blend_color_difference_float, |s1: f32, s2: f32| (s1 - s2).abs());
float_mode!(blend_color_exclusion_float, |s1: f32, s2: f32| {
    0.5 - (2.0 * (s1 - 0.5) * (s2 - 0.5))
});

fn hsv_float_common(
    dst: &mut [f32; 4],
    src1: [f32; 4],
    src2: [f32; 4],
    mix: impl Fn((f32, f32, f32), (f32, f32, f32)) -> (f32, f32, f32),
) {
    let fac = src2[3];
    if fac != 0.0 {
        let mfac = 1.0 - fac;
        let hsv1 = rgb_to_hsv(src1[0], src1[1], src1[2]);
        let hsv2 = rgb_to_hsv(src2[0], src2[1], src2[2]);
        let (h, s, v) = mix(hsv1, hsv2);
        let rgb = hsv_to_rgb(h, s, v);
        for i in 0..3 {
            dst[i] = rgb[i] * fac + src1[i] * mfac;
        }
    } else {
        *dst = src1;
    }
}

/// `blend_color_color_float`.
pub fn blend_color_color_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    hsv_float_common(dst, src1, src2, |(_, _, v1), (h2, s2, _)| (h2, s2, v1));
}

/// `blend_color_hue_float`.
pub fn blend_color_hue_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    hsv_float_common(dst, src1, src2, |(_, s1, v1), (h2, _, _)| (h2, s1, v1));
}

/// `blend_color_saturation_float`.
pub fn blend_color_saturation_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    hsv_float_common(dst, src1, src2, |(h1, s1, v1), (_, s2, _)| {
        (h1, if s1 > EPS_SATURATION { s2 } else { s1 }, v1)
    });
}

/// `blend_color_luminosity_float`.
pub fn blend_color_luminosity_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4]) {
    hsv_float_common(dst, src1, src2, |(h1, s1, _), (_, _, v2)| (h1, s1, v2));
}

/// `blend_color_interpolate_float` — premultiplied lerp.
pub fn blend_color_interpolate_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4], t: f32) {
    let mt = 1.0 - t;
    for i in 0..4 {
        dst[i] = mt * src1[i] + t * src2[i];
    }
}
