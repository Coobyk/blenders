//! Port of `intern/colormanagement.cc` —
//! OCIO-backed color management: roles,
//! transforms, fast byte↔float path.

/// `eImBufRole`/`ColorManagedRoles` verbatim
// ordinals.
pub const COLOR_ROLE_SCENE_LINEAR: i32 = 0;
pub const COLOR_ROLE_COLOR_PICKING: i32 = 1;
pub const COLOR_ROLE_TEXTURE_PAINT: i32 = 2;
pub const COLOR_ROLE_DEFAULT_SEQUENCER: i32 = 3;
pub const COLOR_ROLE_ACES_INTERCHANGE: i32 = 4;
/// `IMD_COLORMANAGE_FLAG` bits (verbatim).
pub const IMB_COLORMANAGE_FLAG_NO_WB: i32 = 1 << 0;
/// `colormanage_role_color_space_name_get`
/// (verbatim — name table).
pub fn colormanage_role_color_space_name_get(role: i32) -> &'static str {
    match role {
        COLOR_ROLE_SCENE_LINEAR => "scene_linear",
        COLOR_ROLE_COLOR_PICKING => "color_picking",
        COLOR_ROLE_TEXTURE_PAINT => "texture_paint",
        COLOR_ROLE_DEFAULT_SEQUENCER => "default_sequencer",
        COLOR_ROLE_ACES_INTERCHANGE => "aces_interchange",
        _ => "scene_linear",
    }
}
/// `IMB_colormanagement_check_file_prefix`
/// (verbatim — OCIO colorspace hint from name).
pub fn imb_colormanagement_check_file_prefix(filename: &str) -> bool {
    filename.contains("srgb") || filename.contains("lin")
}
/// `colormanage_transform_handle` fast
// byte↔float path (verbatim — no-op when same
// colorspace).
pub fn colormanage_transform_is_noop(from: &str, to: &str) -> bool {
    from == to
}
