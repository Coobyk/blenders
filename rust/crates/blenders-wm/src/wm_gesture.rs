//! Port of `intern/wm_gesture.cc` — `wmGesture`
//! lifecycle (new/end/free/remove), rect/lasso/polyline
//! customdata init, draw dispatch, tag_redraw.

/// `WM_GESTURE_*` verbatim (`WM_types.h`).
pub const WM_GESTURE_RECT: i32 = 1;
pub const WM_GESTURE_CROSS_RECT: i32 = 2;
pub const WM_GESTURE_LINES: i32 = 3;
pub const WM_GESTURE_LASSO: i32 = 4;
pub const WM_GESTURE_CIRCLE: i32 = 5;
pub const WM_GESTURE_STRAIGHTLINE: i32 = 6;
pub const WM_GESTURE_POLYLINE: i32 = 7;
/// `GESTURE_MODAL_NOP` verbatim.
pub const GESTURE_MODAL_NOP: i32 = 0;

/// `wmGesture` verbatim fields.
pub struct WmGesture {
    /// `type`.
    pub gesture_type: i32,
    /// `event_type`/`event_modifier`/`event_keymodifier`.
    pub event_type: i32,
    pub event_modifier: i32,
    pub event_keymodifier: i32,
    /// `winrct`.
    pub winrct: crate::wm_subwindow::Rcti,
    /// `user_data.use_free`.
    pub user_data_use_free: bool,
    /// `modal_state`.
    pub modal_state: i32,
    /// `move`.
    pub is_move: bool,
    /// `is_active`/`is_active_prev`.
    pub is_active: bool,
    pub is_active_prev: bool,
    /// `points`/`points_alloc`.
    pub points: i32,
    pub points_alloc: usize,
    /// `mval`.
    pub mval: (i16, i16),
    /// `customdata` — rect (RECT/CROSS/CIRCLE/STRAIGHTLINE),
    /// lasso f32 xy (LINES/LASSO), border i16 xy (POLYLINE).
    pub customdata_rect: Option<crate::wm_subwindow::Rcti>,
    pub customdata_lasso: Option<Vec<f32>>,
    pub customdata_border: Option<Vec<i16>>,
}

/// `WM_gesture_new` (verbatim — addtail + per-type
/// customdata init from drag-start xy minus winrct).
pub fn wm_gesture_new(
    gestures: &mut Vec<WmGesture>,
    event_type: i32,
    event_modifier: i32,
    event_keymodifier: i32,
    region_winrct: crate::wm_subwindow::Rcti,
    gesture_type: i32,
    drag_start_xy: (i32, i32),
) -> usize {
    let xy = drag_start_xy;
    let mut gesture = WmGesture {
        gesture_type,
        event_type,
        event_modifier,
        event_keymodifier,
        winrct: region_winrct,
        user_data_use_free: true,
        modal_state: GESTURE_MODAL_NOP,
        is_move: false,
        is_active: false,
        is_active_prev: false,
        points: 0,
        points_alloc: 0,
        mval: (0, 0),
        customdata_rect: None,
        customdata_lasso: None,
        customdata_border: None,
    };
    if matches!(gesture_type, WM_GESTURE_RECT | WM_GESTURE_CROSS_RECT | WM_GESTURE_CIRCLE | WM_GESTURE_STRAIGHTLINE) {
        let mut rect = crate::wm_subwindow::Rcti::default();
        rect.xmin = xy.0 - gesture.winrct.xmin;
        rect.ymin = xy.1 - gesture.winrct.ymin;
        if gesture_type == WM_GESTURE_CIRCLE {
            /* Caller initializes xmax to radius. */
        } else {
            rect.xmax = xy.0 - gesture.winrct.xmin;
            rect.ymax = xy.1 - gesture.winrct.ymin;
        }
        gesture.customdata_rect = Some(rect);
    } else if gesture_type == WM_GESTURE_LINES || gesture_type == WM_GESTURE_LASSO {
        gesture.points_alloc = 1024;
        let mut lasso = vec![0.0f32; 2 * gesture.points_alloc];
        lasso[0] = (xy.0 - gesture.winrct.xmin) as f32;
        lasso[1] = (xy.1 - gesture.winrct.ymin) as f32;
        gesture.customdata_lasso = Some(lasso);
        gesture.points = 1;
    } else if gesture_type == WM_GESTURE_POLYLINE {
        gesture.points_alloc = 64;
        let mut border = vec![0i16; 2 * gesture.points_alloc];
        border[0] = (xy.0 - gesture.winrct.xmin) as i16;
        border[1] = (xy.1 - gesture.winrct.ymin) as i16;
        gesture.mval = (border[0], border[1]);
        gesture.customdata_border = Some(border);
        gesture.points = 1;
    }
    gestures.push(gesture);
    gestures.len() - 1
}

/// `WM_gesture_end` (verbatim — remlink + free data).
pub fn wm_gesture_end(gestures: &mut Vec<WmGesture>, index: usize) {
    gestures.remove(index);
}

/// `WM_gestures_free_all`/`WM_gestures_remove` (verbatim).
pub fn wm_gestures_free_all(gestures: &mut Vec<WmGesture>) {
    gestures.clear();
}

/// `WM_gesture_is_modal_first` (verbatim — null or
/// !is_active_prev).
pub fn wm_gesture_is_modal_first(gesture: Option<&WmGesture>) -> bool {
    match gesture {
        None => true,
        Some(g) => !g.is_active_prev,
    }
}

/// `wm_gesture_draw` dispatch (verbatim — per-type draw
/// fn after `wmViewport(winrct)`).
pub fn wm_gesture_draw_dispatch(
    gestures: &[WmGesture],
    viewport: &mut dyn FnMut(&crate::wm_subwindow::Rcti),
    draw_rect: &mut dyn FnMut(&WmGesture),
    draw_circle: &mut dyn FnMut(&WmGesture),
    draw_cross: &mut dyn FnMut(&WmGesture),
    draw_lasso: &mut dyn FnMut(&WmGesture, bool),
    draw_line: &mut dyn FnMut(&WmGesture),
    draw_polyline: &mut dyn FnMut(&WmGesture),
) {
    for gt in gestures {
        (viewport)(&gt.winrct);
        match gt.gesture_type {
            WM_GESTURE_RECT => (draw_rect)(gt),
            WM_GESTURE_CIRCLE => (draw_circle)(gt),
            WM_GESTURE_CROSS_RECT => {
                if gt.is_active {
                    (draw_rect)(gt);
                } else {
                    (draw_cross)(gt);
                }
            }
            WM_GESTURE_LINES => (draw_lasso)(gt, false),
            WM_GESTURE_LASSO => (draw_lasso)(gt, true),
            WM_GESTURE_STRAIGHTLINE => (draw_line)(gt),
            WM_GESTURE_POLYLINE => (draw_polyline)(gt),
            _ => {}
        }
    }
}

/// `wm_gesture_tag_redraw` (verbatim —
/// `screen->do_draw_gesture = true`).
pub fn wm_gesture_tag_redraw(screen_do_draw_gesture: &mut bool, screen_present: bool) {
    if screen_present {
        *screen_do_draw_gesture = true;
    }
}
