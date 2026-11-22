/// Port of `usd/intern/usd_archive.cc` — USD
/// archive open (crate/zip/usdc).
/// `usd_open` (verbatim — type sniff).
pub fn usd_open(filepath: &str) -> bool {
    filepath.ends_with(".usd")
        || filepath.ends_with(".usda")
        || filepath.ends_with(".usdc")
        || filepath.ends_with(".usdz")
}
