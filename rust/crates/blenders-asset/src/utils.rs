//! Port of `intern/utils.cc` — path resolve
//! (template + normalize).

/// `resolve_path` (verbatim — cap at max_len, apply
/// template, slash-native + normalize; overflow → raw).
pub fn resolve_path(path: &str, max_len: i64, apply_template: &dyn Fn(&str) -> String, normalize: &dyn Fn(&str) -> String) -> String {
    let len = if max_len == -1 {
        path.len()
    } else {
        (max_len as usize).min(path.len())
    };
    let mut resolved = (normalize)(&(apply_template)(&path[..len]));
    if len != path.len() {
        resolved.push_str(&path[len..]);
    }
    resolved
}
