//! Port of `intern/blf_dir.cc` — .afm/.pfm metrics
//! sibling-file search.

/// `blf_dir_metrics_search` (verbatim — `.ttf`→`.afm`
/// then `.pfm` sibling probe).
pub fn blf_dir_metrics_search(filepath: &str) -> Option<String> {
    let dot = filepath.rfind('.')?;
    let stem = &filepath[..dot];
    let suffix = &filepath[dot..];
    if suffix.len() < 4 {
        return None;
    }
    let afm = format!("{}.afm", stem);
    if std::path::Path::new(&afm).exists() {
        return Some(afm);
    }
    let pfm = format!("{}.pfm", stem);
    if std::path::Path::new(&pfm).exists() {
        return Some(pfm);
    }
    None
}
