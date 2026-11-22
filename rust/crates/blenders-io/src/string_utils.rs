/// Port of `common/intern/string_utils.cc` —
/// IO string helpers (split/trim/tokenize).
/// `split` (verbatim — whitespace tokens).
pub fn split_ws(s: &str) -> Vec<String> {
    s.split_whitespace().map(|t| t.to_string()).collect()
}
/// `split_char` (verbatim — delimiter split).
pub fn split_char(s: &str, delim: char) -> Vec<String> {
    s.split(delim).map(|t| t.to_string()).collect()
}
/// `trim` (verbatim).
pub fn trim(s: &str) -> &str {
    s.trim()
}
