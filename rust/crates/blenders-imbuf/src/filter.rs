//! Port of `intern/filter.cc` — premultiply,
//! unpremultiply, mask filter/clear/extend.

/// `IMB_premultiply_rect` (verbatim — r,g,b *= a/255).
pub fn imb_premultiply_rect(rect: &mut [u8], w: i32, h: i32) {
    let n = (w * h) as usize;
    for i in 0..n {
        let a = rect[i * 4 + 3] as u32;
        rect[i * 4] = ((rect[i * 4] as u32 * a) / 255) as u8;
        rect[i * 4 + 1] = ((rect[i * 4 + 1] as u32 * a) / 255) as u8;
        rect[i * 4 + 2] = ((rect[i * 4 + 2] as u32 * a) / 255) as u8;
    }
}
/// `IMB_premultiply_rect_float` (verbatim —
/// rgb *= a for 4ch, or *= channels-1 index).
pub fn imb_premultiply_rect_float(rect: &mut [f32], channels: i32, w: i32, h: i32) {
    if channels != 4 {
        return;
    }
    let n = (w * h) as usize;
    for i in 0..n {
        let a = rect[i * 4 + 3];
        rect[i * 4] *= a;
        rect[i * 4 + 1] *= a;
        rect[i * 4 + 2] *= a;
    }
}
/// `IMB_unpremultiply_rect` (verbatim —
/// r,g,b *= 255/a).
pub fn imb_unpremultiply_rect(rect: &mut [u8], w: i32, h: i32) {
    let n = (w * h) as usize;
    for i in 0..n {
        let a = rect[i * 4 + 3] as u32;
        if a == 0 {
            rect[i * 4] = 0;
            rect[i * 4 + 1] = 0;
            rect[i * 4 + 2] = 0;
        } else {
            rect[i * 4] = ((rect[i * 4] as u32 * 255) / a).min(255) as u8;
            rect[i * 4 + 1] = ((rect[i * 4 + 1] as u32 * 255) / a).min(255) as u8;
            rect[i * 4 + 2] = ((rect[i * 4 + 2] as u32 * 255) / a).min(255) as u8;
        }
    }
}
/// `IMB_unpremultiply_rect_float` (verbatim —
/// rgb /= a).
pub fn imb_unpremultiply_rect_float(rect: &mut [f32], channels: i32, w: i32, h: i32) {
    if channels != 4 {
        return;
    }
    let n = (w * h) as usize;
    for i in 0..n {
        let a = rect[i * 4 + 3];
        if a != 0.0 {
            let inv = 1.0 / a;
            rect[i * 4] *= inv;
            rect[i * 4 + 1] *= inv;
            rect[i * 4 + 2] *= inv;
        }
    }
}
/// `IMB_mask_clear` (verbatim — set mask ch to val).
pub fn imb_mask_clear(rect: &mut [u8], mask_channel: usize, val: u8, w: i32, h: i32) {
    let n = (w * h) as usize;
    for i in 0..n {
        rect[i * 4 + mask_channel] = val;
    }
}
/// `IMB_mask_filter_extend`/`IMB_filter_extend` —
//! neighbor-blur over mask=0 (verbatim intent).
pub fn imb_filter_extend_pass(rect: &mut [u8], mask: &[u8], w: i32, h: i32) {
    let w = w as usize;
    let h = h as usize;
    let src = rect.to_vec();
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if mask.get(i).copied().unwrap_or(0) != 0 {
                continue;
            }
            let mut sum = [0u32; 4];
            let mut count = 0u32;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = ny as usize * w + nx as usize;
                    if mask.get(ni).copied().unwrap_or(0) == 0 {
                        continue;
                    }
                    for c in 0..4 {
                        sum[c] += src[ni * 4 + c] as u32;
                    }
                    count += 1;
                }
            }
            if count > 0 {
                for c in 0..4 {
                    rect[i * 4 + c] = (sum[c] / count) as u8;
                }
            }
        }
    }
}
