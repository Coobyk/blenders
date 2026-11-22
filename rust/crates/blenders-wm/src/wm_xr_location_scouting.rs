//! Port of `xr/intern/wm_xr_location_scouting.cc` —
//! viewfinder capture: hand path, active action idx,
//! capture matrix, smoothing.

/// `XR_VIEWFINDER_HAND_*`/`XR_VIEWFINDER_MODE_*` verbatim.
pub const XR_VIEWFINDER_HAND_LEFT: i32 = 0;
pub const XR_VIEWFINDER_HAND_RIGHT: i32 = 1;
pub const XR_VIEWFINDER_MODE_LIVE: i32 = 0;
pub const XR_VIEWFINDER_MODE_PLAYBACK: i32 = 1;
pub const XR_VIEWFINDER_MODE_CONFIRM: i32 = 2;

/// `wm_xr_viewfinder_get_hand_user_path` (verbatim).
pub fn wm_xr_viewfinder_get_hand_user_path(viewfinder_hand: i32) -> &'static str {
    match viewfinder_hand {
        XR_VIEWFINDER_HAND_LEFT => "/user/hand/left",
        XR_VIEWFINDER_HAND_RIGHT => "/user/hand/right",
        _ => "",
    }
}

/// `wm_xr_viewfinder_get_active_mode_str` (verbatim).
pub fn wm_xr_viewfinder_get_active_mode_str(active_mode: i32) -> &'static str {
    match active_mode {
        XR_VIEWFINDER_MODE_LIVE => "active_action_live",
        XR_VIEWFINDER_MODE_PLAYBACK => "active_action_playback",
        XR_VIEWFINDER_MODE_CONFIRM => "active_action_confirm",
        _ => "",
    }
}

/// `wm_xr_viewfinder_get_active_action_idx` (verbatim).
pub fn wm_xr_viewfinder_get_active_action_idx(
    active_mode: i32,
    active_action_live: i32,
    active_action_playback: i32,
    active_action_confirm: i32,
) -> i32 {
    match active_mode {
        XR_VIEWFINDER_MODE_LIVE => active_action_live,
        XR_VIEWFINDER_MODE_PLAYBACK => active_action_playback,
        XR_VIEWFINDER_MODE_CONFIRM => active_action_confirm,
        _ => 0,
    }
}

/// `wm_xr_viewfinder_get_pp_overscan` (verbatim — 0..1 →
/// 1..1.5).
pub fn wm_xr_viewfinder_get_pp_overscan(viewfinder_passepartout_overscan: f32) -> f32 {
    viewfinder_passepartout_overscan * 0.5 + 1.0
}

/// `wm_xr_viewfinder_get_capture_mat` (verbatim — grip
/// mat + vertical offset + X-rotation).
pub fn wm_xr_viewfinder_capture_offsets(viewfinder_height: f32, xr_ui_unit_fac: f32) -> f32 {
    const BASE_CONTROLLER_OFFSET: f32 = -0.1;
    let height_offset = (viewfinder_height / 2.0) * xr_ui_unit_fac * -1.0;
    BASE_CONTROLLER_OFFSET + height_offset
}
