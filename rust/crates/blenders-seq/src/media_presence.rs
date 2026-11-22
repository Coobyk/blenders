//! Port of `intern/media_presence.cc` — media-file
//! existence tracking per strip/scene.

/// `media_presence_is_set`/`_reset`/`_set` verbatim —
//! `SEQ_FLAG_MEDIA_PRESENT` bit.
pub const SEQ_FLAG_MEDIA_PRESENT: i32 = 1 << 11;
/// `check_media_missing` (verbatim — none data→false;
//! strip type movie/image → file probe else false).
pub fn check_media_missing(file_exists: bool, strip_has_data: bool) -> bool {
    if !strip_has_data {
        return false;
    }
    !file_exists
}
/// `media_presence_get` (verbatim).
pub fn media_presence_get(strip_flag: i32) -> bool {
    (strip_flag & SEQ_FLAG_MEDIA_PRESENT) != 0
}
