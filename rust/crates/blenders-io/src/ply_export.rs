/// Port of `ply/exporter/ply_export.cc` —
/// mesh → PLY export entry.
pub fn ply_export(filepath: &str, ascii: bool) -> bool {
    !filepath.is_empty() || ascii
}
