/// Port of `usd/intern/usd_reader_utils.cc`
/// — shared read helpers.
/// `get_import_name` (verbatim — prim name).
pub fn usd_get_import_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
