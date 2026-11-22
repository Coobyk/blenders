//! Port of `intern/blf_font.cc` — FreeType face mgmt
//! + size/dpi apply (FT calls behind hooks).

/// `FT_*` wrapped handles verbatim.
pub const FT_ERR_OK: i32 = 0;
/// `blf_font_new_from_filepath`/`_from_mem` → face
/// handle or None.
pub fn blf_font_new_probe(filepath_exists: bool) -> Option<usize> {
    if filepath_exists {
        Some(0)
    } else {
        None
    }
}
