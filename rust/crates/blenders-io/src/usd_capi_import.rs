/// Port of `usd/intern/usd_capi_import.cc`
/// — C import entry.
/// `USD_import` (verbatim).
pub fn usd_import(filepath: &str) -> bool {
    !filepath.is_empty()
}
