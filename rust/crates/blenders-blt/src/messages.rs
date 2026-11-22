//! Port of `intern/messages.cc` — ascii case helpers +
//! locale msg lookup shell.

/// `is_upper_ascii`/`is_lower_ascii`/`is_numeric_ascii`
/// (verbatim).
pub fn is_upper_ascii(c: u8) -> bool {
    (b'A'..=b'Z').contains(&c)
}
pub fn is_lower_ascii(c: u8) -> bool {
    (b'a'..=b'z').contains(&c)
}
pub fn is_numeric_ascii(c: u8) -> bool {
    (b'0'..=b'9').contains(&c)
}
/// `make_lower_ascii`/`make_upper_ascii` (verbatim —
/// in-place, returns changed).
pub fn make_lower_ascii(c: &mut u8) -> bool {
    if is_upper_ascii(*c) {
        *c += b'a' - b'A';
        return true;
    }
    false
}
pub fn make_upper_ascii(c: &mut u8) -> bool {
    if is_lower_ascii(*c) {
        *c -= b'a' - b'A';
        return true;
    }
    false
}

/// `MOMessages` — context→msgid→msgstr map (verbatim
/// lookup semantics).
#[derive(Default)]
pub struct MoMessages {
    /// per-domain: (msgctxt, msgid) → msgstr.
    pub domains: Vec<std::collections::HashMap<(String, String), String>>,
}
impl MoMessages {
    /// `translate` (verbatim — domain lookup, context
    /// then plain msgid).
    pub fn translate(&self, domain: usize, msgctxt: &str, msgid: &str) -> Option<String> {
        let map = self.domains.get(domain)?;
        if let Some(t) = map.get(&(msgctxt.to_string(), msgid.to_string())) {
            return Some(t.clone());
        }
        map.get(&(String::new(), msgid.to_string())).cloned()
    }
}
