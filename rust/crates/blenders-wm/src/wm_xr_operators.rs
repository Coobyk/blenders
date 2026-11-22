//! Port of `xr/intern/wm_xr_operators.cc` — XR operators:
//! navlocks, grab compute/apply, teleport arc, OT regs.

/// `XR_TELEPORT_RAY_*` verbatim.
pub const XR_TELEPORT_RAY_MISS: i32 = 0;
pub const XR_TELEPORT_RAY_HIT: i32 = 1;
pub const XR_TELEPORT_RAY_FALLBACK: i32 = 2;
/// `g_xr_teleportation_arc_num_control_points` verbatim.
pub const XR_TELEPORT_ARC_NUM_CONTROL_POINTS: usize = 24;

fn cross3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn len3(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn mul_v3_fl(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

/// `orient_mat_z_normalized` (verbatim — rebuild x/y
/// around fixed z at original scale).
pub fn orient_mat_z_normalized(r: &mut [[f32; 4]; 4], z_axis: [f32; 3]) {
    let scale = len3([r[0][0], r[0][1], r[0][2]]);
    let r0 = [r[0][0], r[0][1], r[0][2]];
    let mut y_axis = cross3(z_axis, r0);
    let ylen = len3(y_axis);
    if ylen != 0.0 {
        y_axis = mul_v3_fl(y_axis, 1.0 / ylen);
    }
    r[1] = [y_axis[0] * scale, y_axis[1] * scale, y_axis[2] * scale, r[1][3]];
    let r1 = [r[1][0], r[1][1], r[1][2]];
    let mut x_axis = cross3(r1, z_axis);
    let xlen = len3(x_axis);
    if xlen != 0.0 {
        x_axis = mul_v3_fl(x_axis, 1.0 / xlen);
    }
    r[0] = [x_axis[0] * scale, x_axis[1] * scale, x_axis[2] * scale, r[0][3]];
    r[2] = [z_axis[0] * scale, z_axis[1] * scale, z_axis[2] * scale, r[2][3]];
}

/// `wm_xr_navigation_grab_can_do_bimanual`/`is_bimanual_ending`
/// (verbatim).
pub fn grab_can_do_bimanual(actiondata_bimanual: bool, data_bimanual_prev: bool) -> bool {
    actiondata_bimanual && data_bimanual_prev
}
pub fn grab_is_bimanual_ending(actiondata_bimanual: bool, data_bimanual_prev: bool) -> bool {
    !actiondata_bimanual && data_bimanual_prev
}

/// `wm_xr_navigation_grab_is_locked` (verbatim —
/// bimanual needs all 3, one-hand ignores scale).
pub fn navigation_grab_is_locked(loc_lock: bool, rot_lock: bool, scale_lock: bool, bimanual: bool) -> bool {
    if bimanual {
        loc_lock && rot_lock && scale_lock
    } else {
        loc_lock && rot_lock
    }
}

/// `wm_xr_navigation_teleport_is_wall_hit` (verbatim —
/// normal·up < cos30°).
pub fn teleport_is_wall_hit(hit_normal: [f32; 3]) -> bool {
    const MIN_GROUND_DOT: f32 = 0.866_025_4; /* M_SQRT3 / 2 */
    let dot = hit_normal[0] * 0.0 + hit_normal[1] * 0.0 + hit_normal[2] * 1.0;
    dot < MIN_GROUND_DOT
}

/// `wm_xr_navigation_teleport_arc_clip_to_ground`
/// (verbatim — truncate arc at z=0 crossing).
pub fn teleport_arc_clip_to_ground(points: &mut [[f32; 3]], endpoint_idx: &mut usize) -> bool {
    for i in 1..XR_TELEPORT_ARC_NUM_CONTROL_POINTS.min(points.len()) {
        let startpoint = points[i - 1];
        let endpoint = points[i];
        if !(startpoint[2] > 0.0 && endpoint[2] < 0.0) {
            continue;
        }
        let alpha = startpoint[2] / (startpoint[2] - endpoint[2]);
        points[i] = [
            startpoint[0] + (endpoint[0] - startpoint[0]) * alpha,
            startpoint[1] + (endpoint[1] - startpoint[1]) * alpha,
            startpoint[2] + (endpoint[2] - startpoint[2]) * alpha,
        ];
        *endpoint_idx = i;
        return true;
    }
    false
}

/// `WM_OT_xr_*` registrations (verbatim).
pub const XR_OT_REGS: &[(&str, &str, &str)] = &[
    ("WM_OT_xr_session_toggle", "Toggle VR Session", "Toggle VR session"),
    ("WM_OT_xr_navigation_grab", "Navigation Grab", "Navigate the VR scene by grabbing"),
    ("WM_OT_xr_navigation_fly", "Navigation Fly", "Fly through the VR scene"),
    ("WM_OT_xr_navigation_teleport", "Navigation Teleport", "Teleport to a location in the VR scene"),
    ("WM_OT_xr_navigation_reset", "Navigation Reset", "Reset VR navigation"),
    ("WM_OT_xr_navigation_swap_hands", "Navigation Swap Hands", "Swap VR controller hands"),
];
