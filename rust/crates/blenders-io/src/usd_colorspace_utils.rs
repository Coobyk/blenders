/// Port of `usd/intern/usd_colorspace_utils.cc`
/// — colorspace name normalization.
/// `get_color_space` (verbatim — map to USD
/// token).
pub fn usd_get_color_space(colorspace: &str) -> &'static str {
    match colorspace {
        "sRGB" => "sRGB",
        "Linear" | "linear" => "linear",
        _ => "raw",
    }
}
