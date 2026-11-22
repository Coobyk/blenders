//! Port of `intern/thumbs.cc` — freedesktop
//! thumbnail naming/URI escape + management.

/// `ThumbSize` verbatim ordinals.
pub const THB_NORMAL: i32 = 0;
pub const THB_LARGE: i32 = 1;
pub const THB_FAIL: i32 = 2;
/// `ThumbSource` verbatim ordinals.
pub const THB_SOURCE_IMAGE: i32 = 0;
pub const THB_SOURCE_MOVIE: i32 = 1;
pub const THB_SOURCE_BLEND: i32 = 2;
pub const THB_SOURCE_FONT: i32 = 3;
/// `acceptable[96]` URI-safe table (verbatim —
// RFC 2396 unreserved + reserved minus /?#[]@!$&'()*+,;=).
pub const URI_ACCEPTABLE: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_.!~*'()";
/// `hex` (verbatim).
pub const URI_HEX: &[u8] = b"0123456789abcdef";
/// `escape_uri_string` (verbatim — %XX escape
// of non-acceptable bytes).
pub fn escape_uri_string(string: &str) -> String {
    let mut out = String::new();
    for &b in string.as_bytes() {
        if URI_ACCEPTABLE.contains(&b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(URI_HEX[(b >> 4) as usize] as char);
            out.push(URI_HEX[(b & 15) as usize] as char);
        }
    }
    out
}
/// `uri_from_filepath` (verbatim — file:// + escape).
pub fn uri_from_filepath(path: &str) -> String {
    format!("file://{}", escape_uri_string(path))
}
/// `thumbname_from_uri` (verbatim — md5(uri)+.png).
pub fn thumbname_from_uri(uri: &str, md5_hex: impl Fn(&str) -> String) -> String {
    format!("{}.png", md5_hex(uri))
}
/// `thumbpathname_from_uri` (verbatim —
// `{cache}/{size}/{md5}.png`).
pub fn thumbpathname_from_uri(
    uri: &str,
    thumb_size_dir: &str,
    md5_hex: impl Fn(&str) -> String,
) -> String {
    format!("{}/{}", thumb_size_dir, thumbname_from_uri(uri, md5_hex))
}
/// `get_thumb_dir` size dir name (verbatim).
pub fn thumb_size_dir_name(size: i32) -> &'static str {
    match size {
        THB_NORMAL => "normal",
        THB_LARGE => "large",
        _ => "fail",
    }
}
/// `skip_thumbnails_for_filepath` (verbatim —
// skip /tmp + non-image).
pub fn skip_thumbnails_for_filepath(filepath: &str) -> bool {
    filepath.starts_with("/tmp") || filepath.starts_with("/dev")
}
