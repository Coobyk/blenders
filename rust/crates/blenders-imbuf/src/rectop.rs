//! Port of `intern/rectop.cc` — rect blend,
//! copy, crop, fill, clip.

use blenders_math::blend::*;

/// `IMB_BlendMode` verbatim ordinals.
pub const IMB_BLEND_MIX: i32 = 0;
pub const IMB_BLEND_ADD: i32 = 1;
pub const IMB_BLEND_SUB: i32 = 2;
pub const IMB_BLEND_MUL: i32 = 3;
pub const IMB_BLEND_LIGHTEN: i32 = 4;
pub const IMB_BLEND_DARKEN: i32 = 5;
pub const IMB_BLEND_ERASE_ALPHA: i32 = 6;
pub const IMB_BLEND_ADD_ALPHA: i32 = 7;
pub const IMB_BLEND_OVERLAY: i32 = 8;
pub const IMB_BLEND_HARDLIGHT: i32 = 9;
pub const IMB_BLEND_COLORBURN: i32 = 10;
pub const IMB_BLEND_LINEARBURN: i32 = 11;
pub const IMB_BLEND_COLORDODGE: i32 = 12;
pub const IMB_BLEND_SCREEN: i32 = 13;
pub const IMB_BLEND_SOFTLIGHT: i32 = 14;
pub const IMB_BLEND_PINLIGHT: i32 = 15;
pub const IMB_BLEND_VIVIDLIGHT: i32 = 16;
pub const IMB_BLEND_LINEARLIGHT: i32 = 17;
pub const IMB_BLEND_DIFFERENCE: i32 = 18;
pub const IMB_BLEND_EXCLUSION: i32 = 19;
pub const IMB_BLEND_HUE: i32 = 20;
pub const IMB_BLEND_SATURATION: i32 = 21;
pub const IMB_BLEND_LUMINOSITY: i32 = 22;
pub const IMB_BLEND_COLOR: i32 = 23;
pub const IMB_BLEND_INTERPOLATE: i32 = 24;
pub const IMB_BLEND_COPY_RGB: i32 = 1001;
pub const IMB_BLEND_COPY_ALPHA: i32 = 1002;
/// `IMB_blend_color_byte` (verbatim — dispatch).
pub fn imb_blend_color_byte(dst: &mut [u8; 4], src1: [u8; 4], src2: [u8; 4], mode: i32) {
    match mode {
        IMB_BLEND_MIX => blend_color_mix_byte(dst, src1, src2),
        IMB_BLEND_ADD => blend_color_add_byte(dst, src1, src2),
        IMB_BLEND_SUB => blend_color_sub_byte(dst, src1, src2),
        IMB_BLEND_MUL => blend_color_mul_byte(dst, src1, src2),
        IMB_BLEND_LIGHTEN => blend_color_lighten_byte(dst, src1, src2),
        IMB_BLEND_DARKEN => blend_color_darken_byte(dst, src1, src2),
        IMB_BLEND_ERASE_ALPHA => blend_color_erase_alpha_byte(dst, src1, src2),
        IMB_BLEND_ADD_ALPHA => blend_color_add_alpha_byte(dst, src1, src2),
        IMB_BLEND_OVERLAY => blend_color_overlay_byte(dst, src1, src2),
        IMB_BLEND_HARDLIGHT => blend_color_hardlight_byte(dst, src1, src2),
        IMB_BLEND_COLORBURN => blend_color_burn_byte(dst, src1, src2),
        IMB_BLEND_LINEARBURN => blend_color_linearburn_byte(dst, src1, src2),
        IMB_BLEND_COLORDODGE => blend_color_dodge_byte(dst, src1, src2),
        IMB_BLEND_SCREEN => blend_color_screen_byte(dst, src1, src2),
        IMB_BLEND_SOFTLIGHT => blend_color_softlight_byte(dst, src1, src2),
        IMB_BLEND_PINLIGHT => blend_color_pinlight_byte(dst, src1, src2),
        IMB_BLEND_VIVIDLIGHT => blend_color_vividlight_byte(dst, src1, src2),
        IMB_BLEND_LINEARLIGHT => blend_color_linearlight_byte(dst, src1, src2),
        IMB_BLEND_DIFFERENCE => blend_color_difference_byte(dst, src1, src2),
        IMB_BLEND_EXCLUSION => blend_color_exclusion_byte(dst, src1, src2),
        IMB_BLEND_COLOR => blend_color_color_byte(dst, src1, src2),
        IMB_BLEND_HUE => blend_color_hue_byte(dst, src1, src2),
        IMB_BLEND_SATURATION => blend_color_saturation_byte(dst, src1, src2),
        IMB_BLEND_LUMINOSITY => blend_color_luminosity_byte(dst, src1, src2),
        _ => *dst = src1,
    }
}
/// `IMB_blend_color_float` (verbatim — dispatch).
pub fn imb_blend_color_float(dst: &mut [f32; 4], src1: [f32; 4], src2: [f32; 4], mode: i32) {
    match mode {
        IMB_BLEND_MIX => blend_color_mix_float(dst, src1, src2),
        IMB_BLEND_ADD => blend_color_add_float(dst, src1, src2),
        IMB_BLEND_SUB => blend_color_sub_float(dst, src1, src2),
        IMB_BLEND_MUL => blend_color_mul_float(dst, src1, src2),
        IMB_BLEND_LIGHTEN => blend_color_lighten_float(dst, src1, src2),
        IMB_BLEND_DARKEN => blend_color_darken_float(dst, src1, src2),
        IMB_BLEND_ERASE_ALPHA => blend_color_erase_alpha_float(dst, src1, src2),
        IMB_BLEND_ADD_ALPHA => blend_color_add_alpha_float(dst, src1, src2),
        IMB_BLEND_OVERLAY => blend_color_overlay_float(dst, src1, src2),
        IMB_BLEND_HARDLIGHT => blend_color_hardlight_float(dst, src1, src2),
        IMB_BLEND_COLORBURN => blend_color_burn_float(dst, src1, src2),
        IMB_BLEND_LINEARBURN => blend_color_linearburn_float(dst, src1, src2),
        IMB_BLEND_COLORDODGE => blend_color_dodge_float(dst, src1, src2),
        IMB_BLEND_SCREEN => blend_color_screen_float(dst, src1, src2),
        IMB_BLEND_PINLIGHT => blend_color_pinlight_float(dst, src1, src2),
        IMB_BLEND_VIVIDLIGHT => blend_color_vividlight_float(dst, src1, src2),
        IMB_BLEND_LINEARLIGHT => blend_color_linearlight_float(dst, src1, src2),
        IMB_BLEND_DIFFERENCE => blend_color_difference_float(dst, src1, src2),
        IMB_BLEND_EXCLUSION => blend_color_exclusion_float(dst, src1, src2),
        IMB_BLEND_COLOR => blend_color_color_float(dst, src1, src2),
        IMB_BLEND_HUE => blend_color_hue_float(dst, src1, src2),
        IMB_BLEND_SATURATION => blend_color_saturation_float(dst, src1, src2),
        IMB_BLEND_LUMINOSITY => blend_color_luminosity_float(dst, src1, src2),
        _ => *dst = src1,
    }
}
/// `IMB_copy_rect` (verbatim — region copy
/// within same buffer).
#[allow(clippy::too_many_arguments)]
pub fn imb_copy_rect_f32(
    dst: &mut [f32],
    dw: i32,
    dh: i32,
    channels: i32,
    src: &[f32],
    srcx: i32,
    srcy: i32,
    dstx: i32,
    dsty: i32,
    w: i32,
    h: i32,
) {
    for y in 0..h {
        let sy = srcy + y;
        let dy = dsty + y;
        for x in 0..w {
            let sx = srcx + x;
            let dx = dstx + x;
            for c in 0..channels {
                dst[((dy * dw + dx) * channels + c) as usize] =
                    src[((sy * dw + sx) * channels + c) as usize];
            }
        }
    }
}
/// `IMB_rectfill` byte (verbatim — row fill).
pub fn imb_rectfill_u8(rect: &mut [u8], w: i32, h: i32, col: [u8; 4]) {
    let n = (w * h) as usize;
    for i in 0..n {
        rect[i * 4..i * 4 + 4].copy_from_slice(&col);
    }
}
/// `IMB_rectfill` float (verbatim).
pub fn imb_rectfill_f32(rect: &mut [f32], w: i32, h: i32, col: [f32; 4]) {
    let n = (w * h) as usize;
    for i in 0..n {
        rect[i * 4..i * 4 + 4].copy_from_slice(&col);
    }
}
/// `IMB_rectfill_area` (verbatim — sub-rect fill).
#[allow(clippy::too_many_arguments)]
pub fn imb_rectfill_area_f32(
    rect: &mut [f32],
    bufw: i32,
    bufh: i32,
    col: [f32; 4],
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
) {
    let x1 = x1.clamp(0, bufw);
    let x2 = x2.clamp(0, bufw);
    let y1 = y1.clamp(0, bufh);
    let y2 = y2.clamp(0, bufh);
    for y in y1..y2 {
        for x in x1..x2 {
            let i = (y * bufw + x) as usize * 4;
            rect[i..i + 4].copy_from_slice(&col);
        }
    }
}
/// `IMB_rectfill_alpha` (verbatim).
pub fn imb_rectfill_alpha(rect: &mut [f32], w: i32, h: i32, value: f32) {
    let n = (w * h) as usize;
    for i in 0..n {
        rect[i * 4 + 3] = value;
    }
}
