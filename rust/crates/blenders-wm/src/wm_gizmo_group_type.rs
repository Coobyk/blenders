//! Port of `gizmo/intern/wm_gizmo_group_type.cc` —
//! gizmo-group type registry.

/// `WM_gizmogrouptype_find` (verbatim).
pub fn wm_gizmogrouptype_find(
    map: &std::collections::HashMap<String, usize>,
    idname: &str,
) -> Option<usize> {
    if !idname.is_empty() {
        map.get(idname).copied()
    } else {
        None
    }
}
