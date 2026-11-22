//! Port of `gizmo/intern/wm_gizmo_group.cc` — group
//! create/find/free, intersected-gizmo search,
//! drawstep/selected checks, maptype group refs,
//! `GIZMOGROUP_OT_*` registrations.

/// `WM_GIZMO_HIDDEN_SELECT` verbatim.
pub const WM_GIZMO_HIDDEN_SELECT: i32 = 1 << 1;
/// `WM_GIZMOGROUPTYPE_*` verbatim.
pub const WM_GIZMOGROUPTYPE_3D: i32 = 1 << 1;
pub const WM_GIZMOGROUPTYPE_SELECT: i32 = 1 << 2;
pub const WM_GIZMOGROUPTYPE_TOOL_INIT: i32 = 1 << 4;
pub const WM_GIZMOGROUPTYPE_DELAY_REFRESH_FOR_TWEAK: i32 = 1 << 9;
/// `WM_GIZMOGROUP_INIT_*` verbatim.
pub const WM_GIZMOGROUP_INIT_REFRESH: i32 = 1 << 0;

/// `WM_gizmo_cmp_temp_fl`/`_reverse` (verbatim —
/// partial_cmp on temp.f).
pub fn wm_gizmo_cmp_temp_fl(a: f32, b: f32) -> i32 {
    if a < b { -1 } else if a > b { 1 } else { 0 }
}
pub fn wm_gizmo_cmp_temp_fl_reverse(a: f32, b: f32) -> i32 {
    -wm_gizmo_cmp_temp_fl(a, b)
}

/// `wm_gizmogroup_find_intersected_gizmo` walk
/// (verbatim — hidden-select skip, keymap modifier
/// gate, test_select first hit).
pub fn wm_gizmogroup_find_intersected_gizmo(
    gizmos: &[(i32, usize)],
    keymap_uses_modifier: &mut dyn FnMut(usize) -> bool,
    test_select: &mut dyn FnMut(usize) -> i32,
) -> Option<(usize, i32)> {
    for &(flag, gz) in gizmos {
        if (flag & (crate::wm_gizmo::WM_GIZMO_HIDDEN | WM_GIZMO_HIDDEN_SELECT)) != 0 {
            continue;
        }
        if !(keymap_uses_modifier)(gz) {
            continue;
        }
        let part = (test_select)(gz);
        if part != -1 {
            return Some((gz, part));
        }
    }
    None
}

/// `wm_gizmogroup_is_visible_in_drawstep` (verbatim).
pub fn wm_gizmogroup_is_visible_in_drawstep(group_type_flag: i32, drawstep: i32) -> bool {
    match drawstep {
        x if x == crate::wm_gizmo::WM_GIZMOMAP_DRAWSTEP_2D => {
            (group_type_flag & WM_GIZMOGROUPTYPE_3D) == 0
        }
        _ => (group_type_flag & WM_GIZMOGROUPTYPE_3D) != 0,
    }
}

/// `wm_gizmogroup_is_any_selected` (verbatim — SELECT
/// flag + any gizmo SELECT state).
pub fn wm_gizmogroup_is_any_selected(
    group_type_flag: i32,
    gizmo_states: &[i32],
) -> bool {
    if (group_type_flag & WM_GIZMOGROUPTYPE_SELECT) != 0 {
        for &state in gizmo_states {
            if (state & crate::wm_gizmo::WM_GIZMO_STATE_SELECT) != 0 {
                return true;
            }
        }
    }
    false
}

/// `WM_gizmomaptype_group_find_ptr`/`_find` (verbatim —
/// linear ref search).
pub fn wm_gizmomaptype_group_find_ptr(refs: &[(usize, String)], gzgt: usize) -> Option<usize> {
    refs.iter().position(|&(r, _)| r == gzgt)
}
pub fn wm_gizmomaptype_group_find(refs: &[(usize, String)], idname: &str) -> Option<usize> {
    refs.iter().position(|(_, name)| name == idname)
}

/// `WM_gizmo_group_type_poll` (verbatim — owner_id gate
/// + poll cb).
pub fn wm_gizmo_group_type_poll(
    owner_id: &str,
    workspace_owner_id_check: bool,
    has_poll: bool,
    poll_result: bool,
) -> bool {
    if !owner_id.is_empty() && !workspace_owner_id_check {
        return false;
    }
    !has_poll || poll_result
}

/// `GIZMOGROUP_OT_*` registrations (verbatim).
pub const GIZMOGROUP_OT_REGS: &[(&str, &str, &str)] = &[
    ("GIZMOGROUP_OT_gizmo_select", "Gizmo Select", "Select gizmo"),
    ("GIZMOGROUP_OT_gizmo_tweak", "Gizmo Tweak", "Tweak gizmo"),
];
