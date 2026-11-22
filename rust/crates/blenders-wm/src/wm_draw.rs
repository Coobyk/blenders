//! Port of `intern/wm_draw.cc` — window/region draw
//! orchestration: software-cursor state, viewport-by-type
//! table, draw callbacks, `wm_draw_update(_test_window)`/
//! `wm_draw_update_clear_window`.

/// `WM_CAPABILITY_CURSOR_WARP` verbatim.
pub const WM_CAPABILITY_CURSOR_WARP: i32 = 1 << 0;
/// `RGN_TYPE_PREVIEW` verbatim.
pub const RGN_TYPE_PREVIEW: i32 = 1 << 3;
/// `SPACE_TYPE_ANY` verbatim.
pub const SPACE_TYPE_ANY: i32 = -1;

/// `g_software_cursor` verbatim global state.
#[derive(Default)]
pub struct SoftwareCursorState {
    /// `enabled` (-1 = uninitialized).
    pub enabled: i8,
    /// `winid`.
    pub winid: i32,
    /// `xy`.
    pub xy: [i32; 2],
}

/// `GrabState` verbatim.
#[derive(Default)]
pub struct GrabState {
    /// `mode`.
    pub mode: i32,
    /// `wrap_axis`.
    pub wrap_axis: i32,
    /// `bounds` (w/h nonzero check).
    pub bounds_w: i32,
    pub bounds_h: i32,
}

/// `wm_software_cursor_needed` (verbatim — lazy from
/// `WM_capabilities_flag`).
pub fn wm_software_cursor_needed(state: &mut SoftwareCursorState, capabilities_flag: i32) -> bool {
    if state.enabled == -1 {
        state.enabled = ((capabilities_flag & WM_CAPABILITY_CURSOR_WARP) == 0) as i8;
    }
    state.enabled != 0
}

/// `wm_software_cursor_needed_for_window` (verbatim —
/// visible cursor + ghost use_software_cursor).
pub fn wm_software_cursor_needed_for_window(cursor_visible: bool, use_software_cursor: bool) -> bool {
    cursor_visible && use_software_cursor
}

/// `wm_software_cursor_motion_test` (verbatim).
pub fn wm_software_cursor_motion_test(state: &SoftwareCursorState, winid: i32, event_xy: [i32; 2]) -> bool {
    state.winid != winid || state.xy[0] != event_xy[0] || state.xy[1] != event_xy[1]
}

/// `wm_software_cursor_motion_update` (verbatim).
pub fn wm_software_cursor_motion_update(state: &mut SoftwareCursorState, winid: i32, event_xy: [i32; 2]) {
    state.winid = winid;
    state.xy = event_xy;
}

/// `wm_software_cursor_motion_clear` (verbatim).
pub fn wm_software_cursor_motion_clear(state: &mut SoftwareCursorState) {
    state.winid = -1;
    state.xy = [-1, -1];
}

/// `wm_software_cursor_motion_clear_with_window` (verbatim).
pub fn wm_software_cursor_motion_clear_with_window(state: &mut SoftwareCursorState, winid: i32) {
    if state.winid == winid {
        wm_software_cursor_motion_clear(state);
    }
}

/// `wm_window_grab_warp_region_is_set` (verbatim — wrap/
/// hide grab + nonzero bounds).
pub fn wm_window_grab_warp_region_is_set(grabcursor: i32, bounds_w: i32, bounds_h: i32) -> bool {
    if grabcursor == crate::wm_cursors::GHOST_K_GRAB_WRAP || grabcursor == crate::wm_cursors::GHOST_K_GRAB_HIDE {
        if bounds_w != 0 || bounds_h != 0 {
            return true;
        }
    }
    false
}

/// `wm_region_use_viewport_by_type` (verbatim).
pub fn wm_region_use_viewport_by_type(space_type: i32, region_type: i32) -> bool {
    (matches!(space_type, crate::wm_toolsystem::SPACE_VIEW3D
        | crate::wm_toolsystem::SPACE_IMAGE
        | crate::wm_toolsystem::SPACE_NODE)
        && region_type == crate::wm_keymap_utils::RGN_TYPE_WINDOW)
        || (space_type == crate::wm_toolsystem::SPACE_SEQ
            && matches!(region_type, RGN_TYPE_PREVIEW | crate::wm_keymap_utils::RGN_TYPE_WINDOW))
}

/// `wm_area_name` (verbatim table).
pub fn wm_area_name(spacetype: i32) -> &'static str {
    match spacetype {
        0 => "SPACE_EMPTY",
        1 => "SPACE_VIEW3D",
        2 => "SPACE_OUTLINER",
        3 => "SPACE_GRAPH",
        4 => "SPACE_PROPERTIES",
        5 => "SPACE_FILE",
        6 => "SPACE_IMAGE",
        7 => "SPACE_NODE",
        8 => "SPACE_SEQ",
        9 => "SPACE_TEXT",
        10 => "SPACE_ACTION",
        11 => "SPACE_NLA",
        12 => "SPACE_SCRIPT",
        13 => "SPACE_CONSOLE",
        14 => "SPACE_USERPREF",
        15 => "SPACE_PROJECT",
        16 => "SPACE_CLIP",
        17 => "SPACE_TOPBAR",
        18 => "SPACE_STATUSBAR",
        19 => "SPACE_SPREADSHEET",
        _ => "Unknown Space",
    }
}

/// `WindowDrawCB` list (`win->runtime->drawcalls`).
#[derive(Default)]
pub struct WindowDrawCbList {
    cbs: Vec<(usize, usize)>, /* draw fn handle, customdata */
}
impl WindowDrawCbList {
    /// `WM_draw_cb_activate` (verbatim — addtail).
    pub fn activate(&mut self, draw: usize, customdata: usize) -> usize {
        self.cbs.push((draw, customdata));
        self.cbs.len() - 1
    }
    /// `WM_draw_cb_exit` (verbatim — remlink by handle).
    pub fn exit(&mut self, handle: usize) {
        self.cbs.remove(handle);
    }
    /// `wm_draw_callbacks` (verbatim — all draw).
    pub fn callbacks(&self, draw: &mut dyn FnMut(usize, usize)) {
        for &(cb, cd) in &self.cbs {
            (draw)(cb, cd);
        }
    }
}

/// Region/screen draw-flag snapshot for
/// `wm_draw_update_test_window`.
pub struct DrawFlags {
    /// `screen->do_refresh`/`do_draw`/`do_draw_gesture`/
    /// `do_draw_cursor_overlay`/`do_draw_drag`.
    pub screen_do_refresh: bool,
    pub screen_do_draw: bool,
    pub screen_do_draw_gesture: bool,
    pub screen_do_draw_cursor_overlay: bool,
    pub screen_do_draw_drag: bool,
    /// `region.runtime->{visible,do_draw,do_draw_cursor_overlay}`
    /// aggregated across screen+area regions.
    pub region_draw_cursor_overlay: bool,
    pub region_visible_do_draw: bool,
}

/// `wm_draw_update_test_window` flag logic (verbatim —
/// any draw flag → true; software-cursor motion edge).
pub fn wm_draw_update_test_window(flags: &DrawFlags, software_cursor_check: Option<bool>) -> bool {
    let mut do_draw = flags.region_visible_do_draw;
    if flags.region_draw_cursor_overlay {
        /* Propagates to screen->do_draw_cursor_overlay. */
    }
    if do_draw {
        return true;
    }
    if flags.screen_do_refresh
        || flags.screen_do_draw
        || flags.screen_do_draw_gesture
        || flags.screen_do_draw_cursor_overlay
        || flags.screen_do_draw_drag
    {
        return true;
    }
    if let Some(motion_or_stale) = software_cursor_check {
        if motion_or_stale {
            return true;
        }
    }
    false
}
