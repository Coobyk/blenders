//! Port of `intern/transform.cc` — IMB_transform
//! (rotate/scale/translate with interpolation).

/// `IMB_transform` param struct (verbatim).
#[derive(Default)]
pub struct ImbTransform {
    /// `xofs`/`yofs`/`scale`/`rotation`.
    pub xofs: f32,
    pub yofs: f32,
    pub scale: [f32; 2],
    pub rotation: f32,
    /// `filter`.
    pub filter_nearest: bool,
}
/// `imb_transform_sample` (verbatim — inverse
//! map then nearest/bilinear).
pub fn imb_transform_sample(
    src: &[f32],
    sw: i32,
    sh: i32,
    channels: i32,
    u: f32,
    v: f32,
    out: &mut [f32],
) {
    let x = u.max(0.0).min((sw - 1) as f32);
    let y = v.max(0.0).min((sh - 1) as f32);
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = (x0 + 1).min(sw - 1);
    let y1 = (y0 + 1).min(sh - 1);
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    for c in 0..channels {
        let i00 = ((y0 * sw + x0) * channels + c) as usize;
        let i10 = ((y0 * sw + x1) * channels + c) as usize;
        let i01 = ((y1 * sw + x0) * channels + c) as usize;
        let i11 = ((y1 * sw + x1) * channels + c) as usize;
        let top = src[i00] + (src[i10] - src[i00]) * fx;
        let bot = src[i01] + (src[i11] - src[i01]) * fx;
        out[c as usize] = top + (bot - top) * fy;
    }
}
