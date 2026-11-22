//! Port of `intern/strip_lookup.cc` — seqbase
//! lookup by name/frame/channel.

/// `lookup_strip_by_name` (verbatim — first match).
pub fn lookup_strip_by_name<'a>(
    strips: &'a [(usize, String)],
    name: &str,
) -> Option<usize> {
    strips.iter().find(|(_, n)| n == name).map(|(id, _)| *id)
}
/// `lookup_strip_by_index` (verbatim).
pub fn lookup_strip_by_index(strips: &[usize], index: usize) -> Option<usize> {
    strips.get(index).copied()
}
/// `lookup_strip_at_frame` (verbatim — channel +
/// frame inside [start, end)).
pub fn lookup_strip_at_frame(
    strips: &[(usize, i32, f64, f64)],
    channel: i32,
    frame: f64,
) -> Option<usize> {
    strips
        .iter()
        .find(|&(_, ch, start, end)| *ch == channel && frame >= *start && frame < *end)
        .map(|&(id, _, _, _)| id)
}
