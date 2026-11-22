//! Port of `xr/intern/wm_xr_session.cc` — session
//! state machine, base pose, navigation
//! get/set/reset, viewer-scale maintenance.

use crate::wm_xr_draw::{mul_m4, wm_xr_pose_scale_to_imat, wm_xr_pose_scale_to_imat as _unused_imat,
    wm_xr_pose_to_imat as _pose_imat, wm_xr_pose_to_mat, wm_xr_pose_scale_to_mat, GhostXrPose};

/// `wmXrSessionStateEvent` verbatim.
pub const SESSION_STATE_EVENT_START: i32 = 0;
pub const SESSION_STATE_EVENT_RESET_TO_BASE_POSE: i32 = 1;
pub const SESSION_STATE_EVENT_POSITION_TRACKING_TOGGLE: i32 = 2;
pub const SESSION_STATE_EVENT_NONE: i32 = 3;

/// `wmXrSessionState` fields used.
#[derive(Default)]
pub struct WmXrSessionState {
    /// `nav_pose`.
    pub nav_pose: GhostXrPose,
    /// `nav_scale`/`viewer_scale`.
    pub nav_scale: f32,
    pub viewer_scale: f32,
    /// `is_navigation_dirty`/`swap_hands`.
    pub is_navigation_dirty: bool,
    pub swap_hands: bool,
    /// `is_view_data_set`/`force_reset_to_base_pose`.
    pub is_view_data_set: bool,
    pub force_reset_to_base_pose: bool,
    /// `prev_*`.
    pub prev_settings_flag: i32,
    pub prev_base_pose_type: i32,
    pub prev_base_pose_object: usize,
    pub prev_eye_position_ofs: [f32; 3],
    pub prev_view_scale_setting: f32,
    /// `nav_pose_last_actions_sync`/`viewer_scale_last_actions_sync`.
    pub nav_pose_last_actions_sync: GhostXrPose,
    pub viewer_scale_last_actions_sync: f32,
    /// `viewer_pose`/`viewer_mat_base`/`viewer_viewmat`.
    pub viewer_pose: GhostXrPose,
    pub viewer_mat_base: [[f32; 4]; 4],
    pub viewer_viewmat: [[f32; 4]; 4],
    /// `focal_len`.
    pub focal_len: f32,
}

/// `wm_xr_session_draw_data_needs_reset_to_base_pose`
/// (verbatim — force or type/object change w/o
/// pos-tracking).
pub fn session_draw_data_needs_reset_to_base_pose(
    state: &WmXrSessionState,
    use_position_tracking: bool,
    settings_base_pose_type: i32,
    settings_base_pose_object: usize,
) -> bool {
    if state.force_reset_to_base_pose {
        return true;
    }
    !use_position_tracking
        && (state.prev_base_pose_type != settings_base_pose_type
            || state.prev_base_pose_object != settings_base_pose_object)
}

/// `wm_xr_session_state_to_event` (verbatim).
pub fn wm_xr_session_state_to_event(
    state: &WmXrSessionState,
    use_position_tracking: bool,
    settings_base_pose_type: i32,
    settings_base_pose_object: usize,
) -> i32 {
    if !state.is_view_data_set {
        return SESSION_STATE_EVENT_START;
    }
    if session_draw_data_needs_reset_to_base_pose(
        state, use_position_tracking, settings_base_pose_type, settings_base_pose_object,
    ) {
        return SESSION_STATE_EVENT_RESET_TO_BASE_POSE;
    }
    let position_tracking_toggled =
        ((state.prev_settings_flag & crate::wm_xr_draw::XR_SESSION_USE_POSITION_TRACKING) != 0)
            != use_position_tracking;
    if position_tracking_toggled {
        return SESSION_STATE_EVENT_POSITION_TRACKING_TOGGLE;
    }
    SESSION_STATE_EVENT_NONE
}

/// `wm_xr_session_scale_maintain_viewer_pos` (verbatim
/// — keep viewer world pos under scale change).
pub fn wm_xr_session_scale_maintain_viewer_pos(
    state: &mut WmXrSessionState,
    new_scale: f32,
    prev_scale: f32,
) {
    let viewer_location = state.viewer_pose.position;
    let nav_location = state.nav_pose.position;
    let viewer_base_offset: [f32; 3] = [
        (viewer_location[0] - nav_location[0]) / prev_scale,
        (viewer_location[1] - nav_location[1]) / prev_scale,
        (viewer_location[2] - nav_location[2]) / prev_scale,
    ];
    let offset_val = prev_scale - new_scale;
    let view_scaling_offset = [
        viewer_base_offset[0] * offset_val,
        viewer_base_offset[1] * offset_val,
        viewer_base_offset[2] * offset_val,
    ];
    state.nav_pose.position[0] += view_scaling_offset[0];
    state.nav_pose.position[1] += view_scaling_offset[1];
    state.nav_pose.position[2] *= new_scale / prev_scale;
}

/// `wm_xr_session_state_viewer_scale_update` (verbatim —
/// maintain pos on setting change + product scale).
pub fn wm_xr_session_state_viewer_scale_update(
    state: &mut WmXrSessionState,
    view_scale: f32,
    scene_scale: f32,
) {
    if state.prev_view_scale_setting == 0.0 {
        state.prev_view_scale_setting = view_scale;
    }
    if view_scale != state.prev_view_scale_setting {
        wm_xr_session_scale_maintain_viewer_pos(state, view_scale, state.prev_view_scale_setting);
        state.prev_view_scale_setting = view_scale;
    }
    let new_viewer_scale = state.nav_scale * view_scale * scene_scale;
    if state.viewer_scale != new_viewer_scale {
        state.viewer_scale = new_viewer_scale;
        state.is_navigation_dirty = true;
    }
}

/// `WM_xr_session_state_nav_scale_set` (verbatim —
/// clamp + dirty).
pub fn wm_xr_session_state_nav_scale_set(
    state: &mut WmXrSessionState,
    session_exists: bool,
    scale: f32,
    clip_start: f32,
    clip_end: f32,
) {
    if session_exists {
        let s = scale.clamp(clip_start, clip_end);
        state.nav_scale = s;
        state.is_navigation_dirty = true;
    }
}

/// `WM_xr_session_state_navigation_reset` (verbatim).
pub fn wm_xr_session_state_navigation_reset(state: &mut WmXrSessionState) {
    state.nav_pose.position = [0.0; 3];
    state.nav_pose.orientation_quat = [1.0, 0.0, 0.0, 0.0];
    state.nav_scale = 1.0;
    state.viewer_scale = 1.0;
    state.is_navigation_dirty = true;
    state.swap_hands = false;
}

/// `wm_xr_session_state_update` viewer math (verbatim —
/// viewer_pose→mat × base × nav, focal len, prev save).
pub fn wm_xr_session_state_update_compute(
    state: &mut WmXrSessionState,
    local_pose: &GhostXrPose,
    base_pose: &GhostXrPose,
    base_scale: f32,
    eye_position_ofs: [f32; 3],
    use_position_tracking: bool,
    use_absolute_tracking: bool,
    fov_width: f32,
    sensor_width: f32,
    settings_flag: i32,
    settings_base_pose_type: i32,
    settings_base_pose_object: usize,
) {
    let mut viewer_pose = GhostXrPose {
        orientation_quat: local_pose.orientation_quat,
        position: if use_position_tracking { local_pose.position } else { [0.0; 3] },
    };
    if !use_absolute_tracking {
        for i in 0..3 {
            viewer_pose.position[i] -= eye_position_ofs[i];
        }
    }
    let viewer_mat = wm_xr_pose_to_mat(&viewer_pose);
    let base_mat = wm_xr_pose_scale_to_mat(base_pose, base_scale);
    let nav_mat = wm_xr_pose_scale_to_mat(
        &state.nav_pose_last_actions_sync,
        state.viewer_scale_last_actions_sync,
    );
    state.viewer_mat_base = mul_m4(&base_mat, &viewer_mat);
    let viewer_mat = mul_m4(&nav_mat, &state.viewer_mat_base);
    /* `mat4_to_loc_quat`. */
    state.viewer_pose.position = [viewer_mat[3][0], viewer_mat[3][1], viewer_mat[3][2]];
    state.viewer_viewmat = wm_xr_pose_scale_to_imat(
        &state.viewer_pose,
        base_scale * state.viewer_scale_last_actions_sync,
    );
    state.focal_len = 2.0 * (0.5 * sensor_width / (fov_width * 0.5).tan());
    state.prev_eye_position_ofs = eye_position_ofs;
    state.prev_settings_flag = settings_flag;
    state.prev_base_pose_type = settings_base_pose_type;
    state.prev_base_pose_object = settings_base_pose_object;
    state.is_view_data_set = true;
    state.force_reset_to_base_pose = false;
}
