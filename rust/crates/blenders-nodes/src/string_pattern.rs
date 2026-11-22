/// Port of `intern/string_pattern.cc` —
/// `StringPattern` wildcard match (`*`/`?`).
/// `match` (verbatim — glob).
pub fn string_pattern_match(pattern: &str, text: &str) -> bool {
    fn m(p: &[u8], t: &[u8]) -> bool {
        if p.is_empty() {
            return t.is_empty();
        }
        match p[0] {
            b'*' => (0..=t.len()).any(|i| m(&p[1..], &t[i..])),
            b'?' => !t.is_empty() && m(&p[1..], &t[1..]),
            c => !t.is_empty() && t[0] == c && m(&p[1..], &t[1..]),
        }
    }
    m(pattern.as_bytes(), text.as_bytes())
}
