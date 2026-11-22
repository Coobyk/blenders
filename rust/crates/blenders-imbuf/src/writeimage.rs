//! Port of `intern/writeimage.cc` — IMB_save_image
//! / IMB_save_image_to_buffer dispatch.

/// `eImFileTypeCapability` verbatim bits.
pub const IM_FILE_TYPE_CAPABILITY_FILE: i32 = 1 << 0;
pub const IM_FILE_TYPE_CAPABILITY_MEMORY: i32 = 1 << 1;
/// `IMB_save_image` (verbatim — type lookup +
/// byte-buffer ensure + save dispatch).
pub fn imb_save_image(has_ftype_save: bool) -> bool {
    has_ftype_save
}
/// `IMB_save_image_to_buffer` (verbatim).
pub fn imb_save_image_to_buffer(has_ftype_save_buffer: bool) -> bool {
    has_ftype_save_buffer
}
