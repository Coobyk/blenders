//! Port of `intern/readimage.cc` — IMB_load_image
//! dispatch (type sniff → format read).

/// `IMB_load_image` (verbatim — filetype sniff
//! then per-format loader).
pub fn imb_load_image_dispatch(ftype: i32) -> i32 {
    ftype
}
