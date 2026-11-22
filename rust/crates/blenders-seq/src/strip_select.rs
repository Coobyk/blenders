//! Port of `intern/strip_select.cc` — active strip
//! + selection queries.

/// `SEQ_SELECT` verbatim.
pub const SEQ_SELECT: i32 = 1 << 0;
/// `select_active_get`/`select_active_set` verbatim.
pub fn select_active_get(act_strip: Option<usize>) -> Option<usize> {
    act_strip
}
/// `select_active_get_pair` (verbatim — exactly one
/// other selected strip besides active).
pub fn select_active_get_pair(
    strips: &[(usize, i32)],
    act_strip: Option<usize>,
) -> Option<(usize, usize)> {
    let act = act_strip?;
    let mut other = None;
    for &(strip, flag) in strips {
        if (flag & SEQ_SELECT) != 0 && strip != act {
            if other.is_some() {
                return None;
            }
            other = Some(strip);
        }
    }
    other.map(|o| (act, o))
}
/// `select_has_any` (verbatim — any SEQ_SELECT).
pub fn select_has_any(strips: &[(usize, i32)]) -> bool {
    strips.iter().any(|&(_, flag)| (flag & SEQ_SELECT) != 0)
}
