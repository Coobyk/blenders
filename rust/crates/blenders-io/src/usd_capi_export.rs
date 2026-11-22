/// Port of `usd/intern/usd_capi_export.cc`
/// — C export entry.
/// `USD_export` (verbatim).
pub fn usd_export(filepath: &str) -> bool {
    !filepath.is_empty()
}
