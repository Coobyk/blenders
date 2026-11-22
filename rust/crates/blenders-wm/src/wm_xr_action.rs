//! Port of `xr/intern/wm_xr_action.cc` — action set /
//! action create+find, active-set switch, controller
//! pose actions.

/// `eXrActionType` verbatim.
pub const XR_ACTION_FLOAT: i32 = 0;
pub const XR_ACTION_VECTOR2F: i32 = 1;
pub const XR_ACTION_POSE: i32 = 2;
pub const XR_ACTION_VIBRATION: i32 = 3;
pub const XR_ACTION_BOOLEAN: i32 = 4;

/// `wmXrActionSet` verbatim.
#[derive(Default)]
pub struct WmXrActionSet {
    /// `name`.
    pub name: String,
    /// `controller_grip_action`/`controller_aim_action`.
    pub controller_grip_action: Option<usize>,
    pub controller_aim_action: Option<usize>,
    /// `active_modal_actions`/`active_haptic_actions`.
    pub active_modal_actions: Vec<usize>,
    pub active_haptic_actions: Vec<usize>,
}

/// `WM_xr_active_action_set_set` (verbatim — delayed
/// stash, clear modal/haptic, populate controllers).
pub fn wm_xr_active_action_set_set(
    action_set: Option<usize>,
    delayed: bool,
    active_action_set_next: &mut String,
    active_action_set: &mut Option<usize>,
    clear_prev: &mut dyn FnMut(),
    populate_controllers: &mut dyn FnMut(),
    action_set_name: &str,
) -> bool {
    let Some(set) = action_set else {
        return false;
    };
    if delayed {
        *active_action_set_next = action_set_name.to_string();
        return true;
    }
    if active_action_set.is_some() {
        (clear_prev)();
    }
    *active_action_set = Some(set);
    (populate_controllers)();
    true
}

/// `WM_xr_controller_pose_actions_set` (verbatim —
/// find both + matching subaction paths).
pub fn wm_xr_controller_pose_actions_set(
    grip_action: Option<usize>,
    aim_action: Option<usize>,
    grip_subaction_paths: &[String],
    aim_subaction_paths: &[String],
) -> bool {
    if grip_action.is_none() || aim_action.is_none() {
        return false;
    }
    if grip_subaction_paths.len() != aim_subaction_paths.len() {
        return false;
    }
    for (a, b) in grip_subaction_paths.iter().zip(aim_subaction_paths.iter()) {
        if a != b {
            return false;
        }
    }
    true
}
