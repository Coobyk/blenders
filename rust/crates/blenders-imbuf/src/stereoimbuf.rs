//! Port of `intern/stereoimbuf.cc` — stereo3d
//! anaglyph/interlace/side-by-side/top-bottom
//! write+read.

/// `eStereo3dMode` verbatim ordinals.
pub const S3D_DISPLAY_ANAGLYPH: i32 = 0;
pub const S3D_DISPLAY_INTERLACE: i32 = 1;
pub const S3D_DISPLAY_SIDEBYSIDE: i32 = 2;
pub const S3D_DISPLAY_TOPBOTTOM: i32 = 3;
/// `eStereo3dAnaglyphType` verbatim ordinals.
pub const S3D_ANAGLYPH_REDCYAN: i32 = 0;
pub const S3D_ANAGLYPH_GREENMAGENTA: i32 = 1;
pub const S3D_ANAGLYPH_YELLOWBLUE: i32 = 2;
/// `eStereo3dInterlaceType` verbatim ordinals.
pub const S3D_INTERLACE_ROW: i32 = 0;
pub const S3D_INTERLACE_COLUMN: i32 = 1;
pub const S3D_INTERLACE_CHECKERBOARD: i32 = 2;
/// `IMB_stereo3d_write_dimensions` (verbatim —
//! combined size).
pub fn imb_stereo3d_write_dimensions(
    mode: i32,
    is_squeezed: bool,
    x: i32,
    y: i32,
) -> (i32, i32) {
    match mode {
        S3D_DISPLAY_SIDEBYSIDE => (if is_squeezed { x } else { x * 2 }, y),
        S3D_DISPLAY_TOPBOTTOM => (x, if is_squeezed { y } else { y * 2 }),
        _ => (x, y),
    }
}
/// `IMB_stereo3d_read_dimensions` (verbatim —
//! per-eye size).
pub fn imb_stereo3d_read_dimensions(
    mode: i32,
    is_squeezed: bool,
    x: i32,
    y: i32,
) -> (i32, i32) {
    match mode {
        S3D_DISPLAY_SIDEBYSIDE => (if is_squeezed { x } else { x / 2 }, y),
        S3D_DISPLAY_TOPBOTTOM => (x, if is_squeezed { y } else { y / 2 }),
        _ => (x, y),
    }
}
/// `imb_stereo3d_write_anaglyph` (verbatim —
//! per-eye channel merge).
pub fn imb_stereo3d_write_anaglyph_f32(
    left: &[f32],
    right: &[f32],
    w: i32,
    h: i32,
    mode: i32,
) -> Vec<f32> {
    let n = (w * h) as usize;
    let mut out = vec![0.0; n * 4];
    for i in 0..n {
        let l = &left[i * 4..i * 4 + 4];
        let r = &right[i * 4..i * 4 + 4];
        match mode {
            S3D_ANAGLYPH_REDCYAN => {
                out[i * 4] = l[0];
                out[i * 4 + 1] = r[1];
                out[i * 4 + 2] = r[2];
            }
            S3D_ANAGLYPH_GREENMAGENTA => {
                out[i * 4] = r[0];
                out[i * 4 + 1] = l[1];
                out[i * 4 + 2] = r[2];
            }
            _ => {
                out[i * 4] = r[0];
                out[i * 4 + 1] = r[1];
                out[i * 4 + 2] = l[2];
            }
        }
        out[i * 4 + 3] = l[3].max(r[3]);
    }
    out
}
