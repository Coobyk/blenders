//! Port of `intern/action_selection.cc` — deselect all
//! keys on action(s).

/// `action_deselect_keys` (verbatim — deselect keys on
/// every fcurve).
pub fn action_deselect_keys(
    fcurve_handles: &[usize],
    fcurve_deselect_all_keys: &mut dyn FnMut(usize),
) {
    for &fcu in fcurve_handles {
        (fcurve_deselect_all_keys)(fcu);
    }
}

/// `deselect_keys_actions` (verbatim — dedupe via visited
/// set).
pub fn deselect_keys_actions(
    action_handles: &[usize],
    deselect: &mut dyn FnMut(usize),
) {
    let mut visited = std::collections::HashSet::new();
    for &action in action_handles {
        if !visited.insert(action) {
            continue;
        }
        (deselect)(action);
    }
}
