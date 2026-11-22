//! Port of `intern/wm_operators.cc` — operator idname
//! validation, generic select modal, menu/confirm/filesel
//! invokes, popup blocks, `WM_OT_*` registrations.

/// `OP_BL_SEP_STRING`/`OP_BL_SEP_LEN`/`OP_MAX_PY_IDNAME`.
pub const OP_BL_SEP_STRING: &str = "_OT_";
pub const OP_BL_SEP_LEN: usize = 4;
pub const OP_MAX_PY_IDNAME: usize = 63;
/// `OPERATOR_INTERFACE` verbatim.
pub const OPERATOR_INTERFACE: i32 = 1 << 4;
/// `OP_IS_INVOKE`/`USER_REDUCE_MOTION` verbatim.
pub const OP_IS_INVOKE: i32 = 1 << 0;
pub const USER_REDUCE_MOTION: i32 = 1 << 17;

/// `WM_operator_bl_idname_is_valid` (verbatim —
/// `UPPER_OT_lower` with `_`/`0-9`).
pub fn wm_operator_bl_idname_is_valid(idname: &str) -> bool {
    let Some(sep) = idname.find(OP_BL_SEP_STRING) else {
        return false;
    };
    if sep == 0 || sep + OP_BL_SEP_LEN == idname.len() {
        return false;
    }
    for ch in idname[..sep].chars() {
        if !(ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_') {
            return false;
        }
    }
    for ch in idname[sep + OP_BL_SEP_LEN..].chars() {
        if !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_') {
            return false;
        }
    }
    true
}

/// `operator_idname_ok_or_report_impl` (verbatim —
/// lowercase/digit/_ + exactly 1 dot + ≤63).
pub fn operator_idname_ok(idname: &str) -> Result<(), String> {
    let mut dot = 0;
    for (i, ch) in idname.chars().enumerate() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_' {
            continue;
        }
        if ch == '.' {
            if i == 0 || i + 1 == idname.len() {
                return Err(format!("Invalid character at position {}", i));
            }
            dot += 1;
        } else {
            return Err(format!("Invalid character at position {}", i));
        }
    }
    if idname.len() > OP_MAX_PY_IDNAME {
        return Err(format!("Identifier too long, maximum length is {}", OP_MAX_PY_IDNAME));
    }
    if dot != 1 {
        return Err("Must contain 1 '.' character".to_string());
    }
    Ok(())
}

/// `WM_operator_smooth_viewtx_get` (verbatim — INVOKE &&
/// !reduce_motion ? U.smooth_viewtx : 0).
pub fn wm_operator_smooth_viewtx_get(op_flag: i32, uiflag_reduce_motion: bool, smooth_viewtx: i32) -> i32 {
    if (op_flag & OP_IS_INVOKE) != 0 && !uiflag_reduce_motion {
        smooth_viewtx
    } else {
        0
    }
}

/// `WM_generic_select_modal` decision (verbatim — press→
/// exec+wait, release→exec, drag-over-threshold→FINISHED,
/// else RUNNING).
#[derive(PartialEq)]
pub enum GenericSelectAction {
    /// Call `exec` with `wait_to_deselect_others = v`.
    ExecAndWait(bool),
    /// Return FINISHED|PASS_THROUGH (drag exceeded).
    FinishPassThrough,
    /// Return PASS_THROUGH only.
    PassThrough,
    /// Return RUNNING_MODAL|PASS_THROUGH.
    RunningPassThrough,
    /// Add modal handler, RUNNING|PASS_THROUGH.
    WaitForReleaseOrDrag,
}
pub fn wm_generic_select_modal_action(
    init_event_type: i32,
    event_type: i32,
    event_val: i32,
    use_select_on_click: bool,
    drag_delta_over_threshold: bool,
) -> GenericSelectAction {
    if init_event_type == 0 {
        if use_select_on_click {
            return GenericSelectAction::WaitForReleaseOrDrag;
        }
        if event_val == crate::wm_operator_utils::KM_PRESS {
            return GenericSelectAction::ExecAndWait(true);
        }
        return GenericSelectAction::ExecAndWait(false);
    }
    if event_type == init_event_type && event_val == crate::wm_operator_utils::KM_RELEASE {
        return GenericSelectAction::ExecAndWait(false);
    }
    if crate::wm_event_query::ismouse_motion(event_type) {
        if drag_delta_over_threshold {
            return GenericSelectAction::FinishPassThrough;
        }
        return GenericSelectAction::PassThrough;
    }
    GenericSelectAction::RunningPassThrough
}

/// `WM_operator_confirm_or_exec` (verbatim — confirm prop
/// → confirm_ex else exec).
pub fn wm_operator_confirm_or_exec(confirm: bool) -> bool {
    confirm
}

/// `WM_operator_filesel` (verbatim — filepath set → call
/// notest else fileselect+modal).
pub enum FileselAction {
    /// `WM_operator_call_notest`.
    CallExec,
    /// `WM_event_add_fileselect` + RUNNING_MODAL.
    FileselectModal,
}
pub fn wm_operator_filesel(filepath_set: bool) -> FileselAction {
    if filepath_set {
        FileselAction::CallExec
    } else {
        FileselAction::FileselectModal
    }
}

/// `WM_operator_winactive` (verbatim — window && screen).
pub fn wm_operator_winactive(win_present: bool, screen_present: bool) -> bool {
    win_present && screen_present
}

/// `WM_OT_*` registrations in this file (verbatim
/// idname/name/description).
pub const WM_OT_REGS: &[(&str, &str, &str)] = &[
    ("WM_OT_debug_menu", "Debug Menu", "Open a popup to set the debug level"),
    ("WM_OT_operator_defaults", "Operator Defaults", "Set the active operator to its defaults"),
    ("WM_OT_search_menu", "Search Menu", "Pop-up a search menu over all available operators in current context"),
    ("WM_OT_search_operator", "Search Operator", "Search through all the available operators"),
    ("WM_OT_search_single_menu", "Search Single Menu", "Pop-up a search menu over all available operators in current context"),
    ("WM_OT_call_menu", "Call Menu", "Open a predefined menu"),
    ("WM_OT_call_menu_pie", "Call Pie Menu", "Open a predefined pie menu"),
    ("WM_OT_call_panel", "Call Panel", "Open a predefined panel"),
    ("WM_OT_call_asset_shelf_popover", "Call Asset Shelf Popover", "Open the active asset shelf as a popover"),
    ("WM_OT_window_close", "Close Window", "Close the current window"),
    ("WM_OT_window_new", "New Window", "Create a new window"),
    ("WM_OT_window_new_main", "New Main Window", "Create a new main window with a workspace"),
    ("WM_OT_window_fullscreen_toggle", "Toggle Window Fullscreen", "Toggle the current window fullscreen"),
    ("WM_OT_quit_blender", "Quit Blender", "Quit Blender"),
    ("WM_OT_console_toggle", "Toggle System Console", "Toggle system console"),
];
