//! Port of `intern/image_gen.cc` — generated-image buffers: flat
//! color, checker, colored checker (+ grid tint and cell text).
//!
//! `IMB_colormanagement_*` conversions and `BLF_*` font drawing are
//! injected (`PORT_PENDING`); `hsv_to_rgb`/`srgb_to_linearrgb`/
//! `rgba_float_to_uchar` come from `blenders-math`.

use blenders_math::base::power_of_2_max_i;
use blenders_math::color::{hsv_to_rgb, rgba_float_to_uchar, srgb_to_linearrgb};

/// `BKE_image_buf_fill_color`.
pub fn bke_image_buf_fill_color(
    rect_byte: Option<&mut [u8]>,
    rect_float: Option<&mut [f32]>,
    _width: usize,
    height: usize,
    color: [f32; 4],
) {
    if let Some(dst) = rect_float {
        for px in dst.chunks_exact_mut(4).take(height) {
            px.copy_from_slice(&color);
        }
    }
    if let Some(dst) = rect_byte {
        let ccol = rgba_float_to_uchar(color);
        for px in dst.chunks_exact_mut(4).take(height) {
            px.copy_from_slice(&ccol);
        }
    }
}

/// `image_buf_fill_checker_slice` — 32px checker + colored `+` marks.
///
/// `srgb_to_scene_linear` = `IMB_colormanagement_srgb_to_scene_linear_v3`.
fn image_buf_fill_checker_slice(
    mut rect: Option<&mut [u8]>,
    mut rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    offset: usize,
    srgb_to_scene_linear: &dyn Fn([f32; 3]) -> [f32; 3],
) {
    const CHECKER_SIZE: usize = 32;
    const CHECKER_SIZE_HALF: usize = CHECKER_SIZE / 2;

    let dark_linear_color = srgb_to_linearrgb(0.25);
    let bright_linear_color = srgb_to_linearrgb(0.58);

    /* Checkers. */
    let mut idx = 0usize;
    for y in offset..height + offset {
        let mut dark = (-1.0f32).powf((y / CHECKER_SIZE) as f32).floor();
        for x in 0..width {
            if x % CHECKER_SIZE == 0 {
                dark = -dark;
            }
            if let Some(rf) = rect_float.as_deref_mut() {
                let px = &mut rf[idx * 4..idx * 4 + 4];
                if dark > 0.0 {
                    px[0] = dark_linear_color;
                    px[1] = dark_linear_color;
                    px[2] = dark_linear_color;
                    px[3] = 1.0;
                } else {
                    px[0] = bright_linear_color;
                    px[1] = bright_linear_color;
                    px[2] = bright_linear_color;
                    px[3] = 1.0;
                }
            } else if let Some(rb) = rect.as_deref_mut() {
                let px = &mut rb[idx * 4..idx * 4 + 4];
                if dark > 0.0 {
                    px[0] = 64;
                    px[1] = 64;
                    px[2] = 64;
                    px[3] = 255;
                } else {
                    px[0] = 150;
                    px[1] = 150;
                    px[2] = 150;
                    px[3] = 255;
                }
            }
            idx += 1;
        }
    }

    /* 2nd pass, colored `+`. */
    let mut idx = 0usize;
    for y in offset..height + offset {
        let hoffs = 0.125 * (y / CHECKER_SIZE) as f32;
        for x in 0..width {
            let h = 0.125 * (x / CHECKER_SIZE) as f32;
            /* Note that this `+` is not exactly centered since it's a
             * 1px wide line being drawn inside an even sized square —
             * keep as-is since solving requires either using odd
             * sized checkers or double-width lines, see #112653. */
            let test_x = ((x % CHECKER_SIZE) as i32 - CHECKER_SIZE_HALF as i32).abs();
            let test_y = ((y % CHECKER_SIZE) as i32 - CHECKER_SIZE_HALF as i32).abs();
            if test_x < 4 && test_y < 4 && (test_x < 1 || test_y < 1) {
                let hsv_h = (h - hoffs).abs() % 1.0;
                let rgb = hsv_to_rgb(hsv_h, 0.9, 0.9);
                if let Some(rb) = rect.as_deref_mut() {
                    let px = &mut rb[idx * 4..idx * 4 + 4];
                    px[0] = (rgb[0] * 255.0) as u8;
                    px[1] = (rgb[1] * 255.0) as u8;
                    px[2] = (rgb[2] * 255.0) as u8;
                    px[3] = 255;
                }
                if let Some(rf) = rect_float.as_deref_mut() {
                    let lin = srgb_to_scene_linear(rgb);
                    let px = &mut rf[idx * 4..idx * 4 + 4];
                    px[0] = lin[0];
                    px[1] = lin[1];
                    px[2] = lin[2];
                    px[3] = 1.0;
                }
            }
            idx += 1;
        }
    }
}

/// `BKE_image_buf_fill_checker`.
pub fn bke_image_buf_fill_checker(
    rect: Option<&mut [u8]>,
    rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    srgb_to_scene_linear: &dyn Fn([f32; 3]) -> [f32; 3],
) {
    /* `parallel_for(IndexRange(height), 64, …)` — 64-row grain,
     * serial here. */
    let mut rect_owned = rect;
    let mut float_owned = rect_float;
    for y0 in (0..height).step_by(64) {
        let h = (height - y0).min(64);
        let start = y0 * width * 4;
        let end = (y0 + h) * width * 4;
        let rb = rect_owned
            .as_deref_mut()
            .map(|r| &mut r[start..end]);
        let rf = float_owned
            .as_deref_mut()
            .map(|r| &mut r[start..end]);
        image_buf_fill_checker_slice(rb, rf, width, h, y0, srgb_to_scene_linear);
    }
}

/// `BLEND_FLOAT`.
fn blend_float(real: f32, add: f32) -> f32 {
    if real + add <= 1.0 {
        real + add
    } else {
        1.0
    }
}

/// `BLEND_CHAR`.
fn blend_char(real: u8, add: f32) -> u8 {
    let add = (add * 255.0) as i32;
    real.saturating_add(add.clamp(0, 255) as u8).min(255)
}

/// `checker_board_color_fill`.
fn checker_board_color_fill(
    mut rect: Option<&mut [u8]>,
    mut rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    offset: usize,
    total_height: usize,
    rec709_to_scene_linear: &dyn Fn([f32; 3]) -> [f32; 3],
) {
    let mut hue_step = power_of_2_max_i(width as i32 / 8);
    hue_step = hue_step.max(8);

    let mut idx = 0usize;
    for y in offset..height + offset {
        /* Use a number lower than 1.0 else its too bright. */
        let v = 0.1 + (y as f32 * (0.4 / total_height as f32));
        for x in 0..width {
            let h = ((x as i32 / hue_step) as f64 * 1.0 / width as f64 * hue_step as f64) as f32;
            let rgb = hsv_to_rgb(h, 1.0, v);
            if let Some(rb) = rect.as_deref_mut() {
                let px = &mut rb[idx * 4..idx * 4 + 4];
                px[0] = (rgb[0] * 255.0) as u8;
                px[1] = (rgb[1] * 255.0) as u8;
                px[2] = (rgb[2] * 255.0) as u8;
                px[3] = 255;
            }
            if let Some(rf) = rect_float.as_deref_mut() {
                let lin = rec709_to_scene_linear(rgb);
                let px = &mut rf[idx * 4..idx * 4 + 4];
                px[0] = lin[0];
                px[1] = lin[1];
                px[2] = lin[2];
                px[3] = 1.0;
            }
            idx += 1;
        }
    }
}

/// `checker_board_color_tint`.
fn checker_board_color_tint(
    mut rect: Option<&mut [u8]>,
    mut rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    size: usize,
    blend: f32,
    offset: usize,
) {
    let blend_half = blend * 0.5;

    let mut idx = 0usize;
    for y in offset..height + offset {
        for x in 0..width {
            let strong = ((y / size) % 2 == 1 && (x / size) % 2 == 1)
                || ((y / size) % 2 == 0 && (x / size) % 2 == 0);
            let b = if strong { blend } else { blend_half };
            if let Some(rb) = rect.as_deref_mut() {
                let px = &mut rb[idx * 4..idx * 4 + 4];
                px[0] = blend_char(px[0], b);
                px[1] = blend_char(px[1], b);
                px[2] = blend_char(px[2], b);
                px[3] = 255;
            }
            if let Some(rf) = rect_float.as_deref_mut() {
                let px = &mut rf[idx * 4..idx * 4 + 4];
                px[0] = blend_float(px[0], b);
                px[1] = blend_float(px[1], b);
                px[2] = blend_float(px[2], b);
                px[3] = 1.0;
            }
            idx += 1;
        }
    }
}

/// `checker_board_grid_fill`.
fn checker_board_grid_fill(
    mut rect: Option<&mut [u8]>,
    mut rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    blend: f32,
    offset: usize,
) {
    let mut idx = 0usize;
    for y in offset..height + offset {
        for x in 0..width {
            if (y % 32) == 0 || (x % 32) == 0 || x == 0 {
                if let Some(rb) = rect.as_deref_mut() {
                    let px = &mut rb[idx * 4..idx * 4 + 4];
                    px[0] = blend_char(px[0], blend);
                    px[1] = blend_char(px[1], blend);
                    px[2] = blend_char(px[2], blend);
                    px[3] = 255;
                }
                if let Some(rf) = rect_float.as_deref_mut() {
                    let px = &mut rf[idx * 4..idx * 4 + 4];
                    px[0] = blend_float(px[0], blend);
                    px[1] = blend_float(px[1], blend);
                    px[2] = blend_float(px[2], blend);
                    px[3] = 1.0;
                }
            }
            idx += 1;
        }
    }
}

/// `checker_board_text` — per-cell "A1" glyphs via `draw_text`.
///
/// `draw_text(text, pen_x, pen_y, color)` ≙ the 9-call outline +
/// center sequence (`BLF_*` `PORT_PENDING`).
fn checker_board_text(
    rect: &mut Option<&mut [u8]>,
    rect_float: &mut Option<&mut [f32]>,
    width: usize,
    height: usize,
    step: usize,
    outline: i32,
    draw_text: &mut dyn FnMut(&str, i32, i32, [f32; 4], Option<&mut [f32]>, Option<&mut [u8]>, usize, usize),
) {
    let char_array = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let char_num = char_array.len();

    let text_color = [0.0, 0.0, 0.0, 1.0];
    let text_outline = [1.0, 1.0, 1.0, 1.0];

    let mut first_char_index = 0usize;
    let mut y = 0usize;
    while y < height {
        let mut text = [b'A', b'1'];
        text[0] = char_array[first_char_index];

        let mut second_char_index = 27usize;
        let mut x = 0usize;
        while x < width {
            text[1] = char_array[second_char_index];
            let pen_x = x as i32 + 33;
            let pen_y = y as i32 + 44;
            let s = std::str::from_utf8(&text).unwrap();

            /* Terribly crappy outline font! */
            for (dx, dy) in [
                (-outline, 0),
                (outline, 0),
                (0, -outline),
                (0, outline),
                (-outline, -outline),
                (outline, outline),
                (-outline, outline),
                (outline, -outline),
            ] {
                draw_text(
                    s,
                    pen_x + dx,
                    pen_y + dy,
                    text_outline,
                    rect_float.as_deref_mut(),
                    rect.as_deref_mut(),
                    width,
                    height,
                );
            }
            draw_text(
                s,
                pen_x,
                pen_y,
                text_color,
                rect_float.as_deref_mut(),
                rect.as_deref_mut(),
                width,
                height,
            );

            second_char_index = (second_char_index + 1) % char_num;
            x += step;
        }
        first_char_index = (first_char_index + 1) % char_num;
        y += step;
    }
}

/// `checker_board_color_prepare_slice`.
#[allow(clippy::too_many_arguments)]
fn checker_board_color_prepare_slice(
    rect: Option<&mut [u8]>,
    rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    offset: usize,
    total_height: usize,
    rec709_to_scene_linear: &dyn Fn([f32; 3]) -> [f32; 3],
) {
    /* Split borrows across sequential calls via `Option.take`. */
    let mut rect = rect;
    let mut rect_float = rect_float;
    checker_board_color_fill(
        rect.as_deref_mut(),
        rect_float.as_deref_mut(),
        width,
        height,
        offset,
        total_height,
        rec709_to_scene_linear,
    );
    for &(size, blend) in &[(1usize, 0.03f32), (4, 0.05), (32, 0.07), (128, 0.15)] {
        checker_board_color_tint(
            rect.as_deref_mut(),
            rect_float.as_deref_mut(),
            width,
            height,
            size,
            blend,
            offset,
        );
    }
    checker_board_grid_fill(
        rect, rect_float, width, height, 1.0 / 4.0, offset,
    );
}

/// `BKE_image_buf_fill_checker_color`.
#[allow(clippy::too_many_arguments)]
pub fn bke_image_buf_fill_checker_color(
    rect: Option<&mut [u8]>,
    rect_float: Option<&mut [f32]>,
    width: usize,
    height: usize,
    rec709_to_scene_linear: &dyn Fn([f32; 3]) -> [f32; 3],
    draw_text: &mut dyn FnMut(&str, i32, i32, [f32; 4], Option<&mut [f32]>, Option<&mut [u8]>, usize, usize),
    float_rgba_srgb_to_linear: &dyn Fn(&mut [f32]),
) {
    let mut rect = rect;
    let mut rect_float = rect_float;
    for y0 in (0..height).step_by(64) {
        let h = (height - y0).min(64);
        let start = y0 * width * 4;
        let end = (y0 + h) * width * 4;
        let rb = rect.as_deref_mut().map(|r| &mut r[start..end]);
        let rf = rect_float.as_deref_mut().map(|r| &mut r[start..end]);
        checker_board_color_prepare_slice(
            rb, rf, width, h, y0, height, rec709_to_scene_linear,
        );
    }

    checker_board_text(
        &mut rect, &mut rect_float, width, height, 128, 2, draw_text,
    );

    if let Some(rf) = rect_float {
        /* It is easier to fill the buffer in sRGB and linearize it
         * afterwards. */
        float_rgba_srgb_to_linear(rf);
    }
}
