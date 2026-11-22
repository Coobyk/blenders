//! Port of `intern/rotate.cc` — IMB_flipy/
//! IMB_mirror_y and arbitrary rotation entry.

/// `IMB_flipy` (verbatim — swap rows top↔bottom).
pub fn imb_flipy(ibuf: &mut crate::allocimbuf::ImBuf) {
    let w = ibuf.x as usize;
    let ch = ibuf.channels.max(1) as usize;
    let stride = w * ch;
    let h = ibuf.y as usize;
    for y in 0..h / 2 {
        let top = y * stride;
        let bot = (h - 1 - y) * stride;
        for i in 0..stride {
            ibuf.byte_data.swap(top + i, bot + i);
        }
    }
    let fstride = w * ch;
    for y in 0..h / 2 {
        let top = y * fstride;
        let bot = (h - 1 - y) * fstride;
        for i in 0..fstride {
            ibuf.float_data.swap(top + i, bot + i);
        }
    }
}
/// `IMB_rotate_float`/`IMB_rotate_byte` shells
//! (nearest + bilinear in transform.rs).
pub enum ImbInterp {
    Nearest,
    Bilinear,
    Bicubic,
}
