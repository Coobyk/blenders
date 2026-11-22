/// Port of `usd/intern/usd_asset_utils.cc` —
/// USD asset path resolve/copy helpers.
/// `usd_get_absolute_filepath` (verbatim —
/// resolve relative to stage dir).
pub fn usd_get_absolute_filepath(path: &str, stage_dir: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("{}/{}", stage_dir, path)
    }
}
