/// Port of `space_file/file_utils.cc` —
/// file-space helpers (dir list, name
/// sanitize).
/// `file_path_sanitize` (verbatim — strip
/// slashes).
pub fn file_path_sanitize(path: &str) -> String {
    path.trim_matches('/').to_string()
}
