//! Port of `src/blendthumb_extract.cc` — thumbnail
//! extraction from a .blend (BHead test + dim clamp).

/// `blendthumb_create_thumb_from_file` result.
#[derive(Default)]
pub struct ExtractedThumb {
    /// `width`/`height`/`data`.
    pub width: i32,
    pub height: i32,
    pub data: Vec<u8>,
}
/// `blendthumb_create_thumb_from_file` validation
/// (verbatim — BHead TEST + x*y*4 size match).
pub fn thumb_extract_validate(
    bhead_name: &[u8],
    reported_len: usize,
    width: i32,
    height: i32,
) -> bool {
    if bhead_name != b"TEST" {
        return false;
    }
    reported_len == (width * height * 4) as usize
}
/// `extract_png_from_blend_file` status flow (verbatim).
pub fn thumb_extract_decide(
    file_open_ok: bool,
    thumb_err: i32,
    png_opt_ok: bool,
    write_ok: bool,
) -> i32 {
    if !file_open_ok {
        return crate::blendthumb_png::BT_FILE_ERR;
    }
    if thumb_err != crate::blendthumb_png::BT_OK {
        return thumb_err;
    }
    if !png_opt_ok {
        return crate::blendthumb_png::BT_ERROR;
    }
    if !write_ok {
        return crate::blendthumb_png::BT_FILE_ERR;
    }
    crate::blendthumb_png::BT_OK
}
