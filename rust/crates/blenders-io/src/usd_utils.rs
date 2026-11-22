/// Port of `usd/intern/usd_utils.cc` — path
/// sanitize, pxr name, timecode helpers.
/// `get_safe_usd_name` (verbatim — replace
/// invalid chars with _).
pub fn get_safe_usd_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
/// `make_safe_name` (verbatim — leading digit
/// → prepend _).
pub fn usd_make_safe_name(name: &str) -> String {
    let n = get_safe_usd_name(name);
    if n.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{}", n)
    } else {
        n
    }
}
