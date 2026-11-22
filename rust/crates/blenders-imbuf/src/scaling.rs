//! Port of `intern/scaling.cc` — box/nearest/
//! bilinear scale.

/// `IMBScaleFilter` verbatim ordinals.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ImbScaleFilter {
    Nearest,
    Box,
    Bilinear,
    Bicubic,
}
/// `scale_nearest` (verbatim — index pick).
pub fn scale_nearest(
    src: &[f32],
    sw: i32,
    sh: i32,
    channels: i32,
    dw: i32,
    dh: i32,
) -> Vec<f32> {
    let mut out = vec![0.0; (dw * dh * channels) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let sx = (x as i64 * sw as i64 / dw as i64) as i32;
            let sy = (y as i64 * sh as i64 / dh as i64) as i32;
            for c in 0..channels {
                out[((y * dw + x) * channels + c) as usize] =
                    src[((sy * sw + sx) * channels + c) as usize];
            }
        }
    }
    out
}
/// `scale_bilinear` (verbatim — fractional blend).
pub fn scale_bilinear(
    src: &[f32],
    sw: i32,
    sh: i32,
    channels: i32,
    dw: i32,
    dh: i32,
) -> Vec<f32> {
    let mut out = vec![0.0; (dw * dh * channels) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let u = (x as f32 + 0.5) * sw as f32 / dw as f32 - 0.5;
            let v = (y as f32 + 0.5) * sh as f32 / dh as f32 - 0.5;
            let x0 = u.floor().max(0.0) as i32;
            let y0 = v.floor().max(0.0) as i32;
            let x1 = (x0 + 1).min(sw - 1);
            let y1 = (y0 + 1).min(sh - 1);
            let fx = (u - x0 as f32).clamp(0.0, 1.0);
            let fy = (v - y0 as f32).clamp(0.0, 1.0);
            for c in 0..channels {
                let i00 = ((y0 * sw + x0) * channels + c) as usize;
                let i10 = ((y0 * sw + x1) * channels + c) as usize;
                let i01 = ((y1 * sw + x0) * channels + c) as usize;
                let i11 = ((y1 * sw + x1) * channels + c) as usize;
                let top = src[i00] + (src[i10] - src[i00]) * fx;
                let bot = src[i01] + (src[i11] - src[i01]) * fx;
                out[((y * dw + x) * channels + c) as usize] = top + (bot - top) * fy;
            }
        }
    }
    out
}
/// `imb_scale_box` (verbatim — area average).
pub fn imb_scale_box(
    src: &[f32],
    sw: i32,
    sh: i32,
    channels: i32,
    dw: i32,
    dh: i32,
) -> Vec<f32> {
    let mut out = vec![0.0; (dw * dh * channels) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let x0 = x as i64 * sw as i64 / dw as i64;
            let x1 = ((x + 1) as i64 * sw as i64 / dw as i64).max(x0 + 1);
            let y0 = y as i64 * sh as i64 / dh as i64;
            let y1 = ((y + 1) as i64 * sh as i64 / dh as i64).max(y0 + 1);
            let mut sum = vec![0.0f64; channels as usize];
            let mut count = 0u64;
            for sy in y0..y1 {
                for sx in x0..x1 {
                    for c in 0..channels {
                        sum[c as usize] += src[((sy as i32 * sw + sx as i32) * channels + c) as usize] as f64;
                    }
                    count += 1;
                }
            }
            for c in 0..channels {
                out[((y * dw + x) * channels + c) as usize] = (sum[c as usize] / count as f64) as f32;
            }
        }
    }
    out
}
/// `IMB_scale` dispatch (verbatim — filter pick).
pub fn imb_scale(
    src: &[f32],
    sw: i32,
    sh: i32,
    channels: i32,
    dw: i32,
    dh: i32,
    filter: ImbScaleFilter,
) -> Vec<f32> {
    match filter {
        ImbScaleFilter::Nearest => scale_nearest(src, sw, sh, channels, dw, dh),
        ImbScaleFilter::Box => imb_scale_box(src, sw, sh, channels, dw, dh),
        _ => scale_bilinear(src, sw, sh, channels, dw, dh),
    }
}
