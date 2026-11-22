//! Port of `intern/conversion.cc` — byte↔float
//! buffer conversion + sRGB + BW + saturation.

/// `IMB_alpha_affects_rgb` (verbatim — 4ch +
/// not premul).
pub fn imb_alpha_affects_rgb(channels: i32, alpha_premul: bool) -> bool {
    channels == 4 && !alpha_premul
}
/// sRGB→linear lookup (verbatim table entry).
pub fn imb_srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}
/// linear→sRGB (verbatim).
pub fn imb_linear_to_srgb(c: f32) -> f32 {
    if c <= 0.0031308 {
        c * 12.92
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}
/// `IMB_buffer_byte_from_float` (verbatim —
/// float→uchar with colorspace + dither).
pub fn imb_buffer_byte_from_float(
    src: &[f32],
    channels: i32,
    use_premul: bool,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(src.len());
    for (i, &f) in src.iter().enumerate() {
        let is_alpha = channels == 4 && i % 4 == 3;
        let v = if is_alpha {
            f.clamp(0.0, 1.0) * 255.0
        } else {
            let c = if use_premul { f } else { f };
            imb_linear_to_srgb(c.clamp(0.0, 1.0)) * 255.0
        };
        out.push(v.round().clamp(0.0, 255.0) as u8);
    }
    out
}
/// `IMB_buffer_float_from_byte` (verbatim —
/// uchar→float sRGB decode).
pub fn imb_buffer_float_from_byte(src: &[u8], channels: i32) -> Vec<f32> {
    src.iter()
        .enumerate()
        .map(|(i, &b)| {
            let is_alpha = channels == 4 && i % 4 == 3;
            if is_alpha {
                b as f32 / 255.0
            } else {
                imb_srgb_to_linear(b as f32 / 255.0)
            }
        })
        .collect()
}
/// `IMB_color_to_bw` (verbatim — luma weights
/// BT.709).
pub fn imb_color_to_bw(r: f32, g: f32, b: f32) -> f32 {
    r * 0.2126 + g * 0.7152 + b * 0.0722
}
/// `IMB_saturation` (verbatim — lerp to luma).
pub fn imb_saturation_pixel(r: f32, g: f32, b: f32, a: f32, sat: f32) -> [f32; 4] {
    let luma = imb_color_to_bw(r, g, b);
    [
        luma + (r - luma) * sat,
        luma + (g - luma) * sat,
        luma + (b - luma) * sat,
        a,
    ]
}
