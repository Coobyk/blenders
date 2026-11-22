//! Port of `gizmo/intern/wm_gizmo_type.cc` — gizmo type
//! registry (idname-keyed append/find/remove).

/// `WM_gizmotype_find` (verbatim — map lookup, quiet).
pub fn wm_gizmotype_find<'a>(
    map: &'a std::collections::HashMap<String, usize>,
    idname: &str,
) -> Option<usize> {
    if !idname.is_empty() {
        map.get(idname).copied()
    } else {
        None
    }
}

/// `WM_gizmotype_append__end` (verbatim — insert).
pub fn wm_gizmotype_append_end(map: &mut std::collections::HashMap<String, usize>, idname: String, handle: usize) {
    map.insert(idname, handle);
}

/// `WM_gizmotype_remove` (verbatim — present → remove).
pub fn wm_gizmotype_remove(map: &mut std::collections::HashMap<String, usize>, idname: &str) -> Option<usize> {
    map.remove(idname)
}
