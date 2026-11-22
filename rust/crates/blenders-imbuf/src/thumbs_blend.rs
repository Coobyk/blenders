//! Port of `intern/thumbs_blend.cc` — .blend
//! file thumbnail extraction (BHead scan → PNG).

/// `imb_create_thumbnail_from_blend` shell —
/// uses `blenders-thumb` `blendthumb` for the
/// actual TEST block decode.
pub fn imb_create_thumbnail_from_blend(file_data: &[u8]) -> Option<Vec<u8>> {
    if file_data.len() < 12 {
        return None;
    }
    if !file_data.starts_with(b"BLENDER") {
        return None;
    }
    None
}
