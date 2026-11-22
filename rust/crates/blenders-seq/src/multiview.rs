//! Port of `intern/multiview.cc` — multiview
//! filename suffix + file counting.

/// `eImageFormat_ViewsFormat` verbatim ordinals.
pub const R_IMF_VIEWS_INDIVIDUAL: i32 = 0;
pub const R_IMF_VIEWS_STEREO_3D: i32 = 1;
pub const R_IMF_VIEWS_MULTIVIEW: i32 = 2;

/// `seq_multiview_num_files_get` (verbatim —
/// combined→1, individual→num_views).
pub fn seq_multiview_num_files_get(views_format: i32, num_views: i32) -> i32 {
    match views_format {
        R_IMF_VIEWS_STEREO_3D | R_IMF_VIEWS_MULTIVIEW => 1,
        R_IMF_VIEWS_INDIVIDUAL => num_views,
        _ => 1,
    }
}
/// `seq_multiview_name` (verbatim —
/// `{prefix}{suffix}{ext}`).
pub fn seq_multiview_name(prefix: &str, suffix: &str, ext: &str) -> String {
    format!("{}{}{}", prefix, suffix, ext)
}
/// `seq_multiview_view_filepath_get` (verbatim —
/// prefix probe + suffix join; empty prefix → None).
pub fn seq_multiview_view_filepath_get(
    prefix: &str,
    suffix: &str,
    ext: &str,
) -> Option<String> {
    if prefix.is_empty() {
        return None;
    }
    Some(format!("{}{}{}", prefix, suffix, ext))
}
