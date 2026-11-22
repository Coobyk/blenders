/// Port of `common/intern/path_util.cc` —
/// export path_reference (relative or abs).

/// `path_reference` (verbatim — //relative
// when `use_relative`, else absolute).
pub fn path_reference(filepath: &str, use_relative: bool) -> String {
    if use_relative {
        format!("//{}", filepath)
    } else {
        filepath.to_string()
    }
}
