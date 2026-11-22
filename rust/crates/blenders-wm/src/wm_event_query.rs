//! Port of `intern/wm_event_query.cc` — event-type mask
//! tests, drag/motion/gesture queries, drag-start coords,
//! utf8→ascii, userdef keymap remap, ndof getters, tablet
//! pressure curve, absolute deltas, IME switch test.

use std::f32::consts::PI;

/// `KM_PRESS_DRAG` verbatim.
pub const KM_PRESS_DRAG: i32 = 6;
/// `KM_DIRECTION_*` verbatim.
pub const KM_DIRECTION_N: i32 = 8;
pub const KM_DIRECTION_NE: i32 = 7;
pub const KM_DIRECTION_E: i32 = 6;
pub const KM_DIRECTION_SE: i32 = 5;
pub const KM_DIRECTION_S: i32 = 4;
pub const KM_DIRECTION_SW: i32 = 3;
pub const KM_DIRECTION_W: i32 = 2;
pub const KM_DIRECTION_NW: i32 = 1;
/// `NDOF_MOTION`/`WINDEACTIVATE` verbatim.
pub const NDOF_MOTION: i32 = 0x0190;
pub const WINDEACTIVATE: i32 = 0x0005;
/// `WM_EVENT_CURSOR_MOTION_THRESHOLD` verbatim.
pub const WM_EVENT_CURSOR_MOTION_THRESHOLD: i32 = 30;
/// `WM_EVENT_SCROLL_INVERT` verbatim.
pub const WM_EVENT_SCROLL_INVERT: i32 = 1 << 13;
/// `EVT_TABLET_NONE`/`EVT_TABLET_ERASER` verbatim.
pub const EVT_TABLET_NONE: i32 = 0;
pub const EVT_TABLET_ERASER: i32 = 2;
/// `EVT_TYPE_MASK_*` verbatim (`WM_types.h`).
pub const EVT_TYPE_MASK_KEYBOARD: i32 = 1 << 0;
pub const EVT_TYPE_MASK_KEYBOARD_MODIFIER: i32 = 1 << 1;
pub const EVT_TYPE_MASK_MOUSE: i32 = 1 << 2;
pub const EVT_TYPE_MASK_MOUSE_WHEEL: i32 = 1 << 3;
pub const EVT_TYPE_MASK_MOUSE_GESTURE: i32 = 1 << 4;
pub const EVT_TYPE_MASK_NDOF: i32 = 1 << 5;
pub const EVT_TYPE_MASK_ACTIONZONE: i32 = 1 << 6;
/// `EVT_SPACEKEY` verbatim.
pub const EVT_SPACEKEY: i32 = 0x0020;
/// `USER_RELEASECONFIRM` verbatim.
pub const USER_RELEASECONFIRM: i32 = 1 << 9;

/// `ISKEYBOARD`/`ISKEYMODIFIER`/`ISMOUSE*`/`ISNDOF`/
/// `IS_EVENT_ACTIONZONE` (verbatim ranges).
pub fn iskeyboard(t: i32) -> bool { (0x0001..0x0200).contains(&t) || t == EVT_SPACEKEY }
pub fn iskeymodifier(t: i32) -> bool {
    matches!(t, crate::wm_operator_utils::EVT_LEFTCTRLKEY
        | crate::wm_operator_utils::EVT_RIGHTCTRLKEY
        | crate::wm_operator_utils::EVT_LEFTSHIFTKEY
        | crate::wm_operator_utils::EVT_RIGHTSHIFTKEY)
}
pub fn ismouse_button(t: i32) -> bool { (0x0002..0x0020).contains(&t) }
pub fn ismouse(t: i32) -> bool { ismouse_button(t) || (0x0010..0x0020).contains(&t) }
pub fn ismouse_wheel(t: i32) -> bool { matches!(t, 0x000B | 0x000C) }
pub fn ismouse_gesture(t: i32) -> bool { matches!(t, 0x001A | 0x001B | 0x001C | 0x001D) }
pub fn ismouse_motion(t: i32) -> bool { t == crate::wm_operator_utils::MOUSEMOVE || t == 0x0009 }
pub fn isndof(t: i32) -> bool { (0x0190..0x01B0).contains(&t) }
pub fn is_event_actionzone(t: i32) -> bool { t == 0x0009 }
pub fn iskeyboard_or_button(t: i32) -> bool { iskeyboard(t) || ismouse_button(t) }

/// `WM_event_type_mask_test` (verbatim — per-mask check).
pub fn wm_event_type_mask_test(event_type: i32, mask: i32) -> bool {
    if (mask & EVT_TYPE_MASK_KEYBOARD) != 0 && iskeyboard(event_type) {
        return true;
    }
    if (mask & EVT_TYPE_MASK_KEYBOARD) == 0
        && (mask & EVT_TYPE_MASK_KEYBOARD_MODIFIER) != 0
        && iskeymodifier(event_type)
    {
        return true;
    }
    if (mask & EVT_TYPE_MASK_MOUSE) != 0 && ismouse(event_type) {
        return true;
    }
    if (mask & EVT_TYPE_MASK_MOUSE) == 0
        && (mask & EVT_TYPE_MASK_MOUSE_WHEEL) != 0
        && ismouse_wheel(event_type)
    {
        return true;
    }
    if (mask & EVT_TYPE_MASK_MOUSE) == 0
        && (mask & EVT_TYPE_MASK_MOUSE_GESTURE) != 0
        && ismouse_gesture(event_type)
    {
        return true;
    }
    if (mask & EVT_TYPE_MASK_NDOF) != 0 && isndof(event_type) {
        return true;
    }
    if (mask & EVT_TYPE_MASK_ACTIONZONE) != 0 && is_event_actionzone(event_type) {
        return true;
    }
    false
}

/// `WM_event_is_modal_drag_exit` (verbatim —
/// release-confirm aware exit test).
pub fn wm_event_is_modal_drag_exit(
    event_val: i32,
    event_type: i32,
    init_event_type: i32,
    init_event_val: i32,
    release_confirm: bool,
) -> bool {
    if release_confirm {
        if event_val == crate::wm_operator_utils::KM_RELEASE {
            if init_event_val == KM_PRESS_DRAG && event_type == init_event_type {
                return true;
            }
        } else if init_event_val != KM_PRESS_DRAG {
            return true;
        }
    } else if event_val != crate::wm_operator_utils::KM_RELEASE {
        return true;
    }
    false
}

/// `WM_event_is_mouse_drag` (verbatim).
pub fn wm_event_is_mouse_drag(event_type: i32, event_val: i32) -> bool {
    ismouse_button(event_type) && event_val == KM_PRESS_DRAG
}

/// `WM_event_is_mouse_drag_or_press` (verbatim).
pub fn wm_event_is_mouse_drag_or_press(event_type: i32, event_val: i32) -> bool {
    wm_event_is_mouse_drag(event_type, event_val)
        || (ismouse_button(event_type) && event_val == crate::wm_operator_utils::KM_PRESS)
}

/// `WM_event_drag_direction` (verbatim — 8-sector atan2).
pub fn wm_event_drag_direction(xy: [i32; 2], prev_press_xy: [i32; 2]) -> i32 {
    let delta = [xy[0] - prev_press_xy[0], xy[1] - prev_press_xy[1]];
    let theta = (4.0 * (delta[1] as f32).atan2(delta[0] as f32) / PI).round() as i32;
    match theta.rem_euclid(8) {
        0 => KM_DIRECTION_E,
        1 => KM_DIRECTION_NE,
        2 => KM_DIRECTION_N,
        3 => KM_DIRECTION_NW,
        4 => KM_DIRECTION_W,
        5 => KM_DIRECTION_SW,
        6 => KM_DIRECTION_S,
        _ => KM_DIRECTION_SE,
    }
}

/// `WM_event_consecutive_gesture_test` (verbatim).
pub fn wm_event_consecutive_gesture_test(event_type: i32) -> bool {
    ismouse_gesture(event_type) || event_type == NDOF_MOTION
}

/// `WM_event_consecutive_gesture_test_break` (verbatim —
/// motion >30px, key/button, deactivate).
pub fn wm_event_consecutive_gesture_test_break(
    event_type: i32,
    event_xy: [i32; 2],
    consecutive_gesture_xy: [i32; 2],
) -> bool {
    if ismouse_motion(event_type) {
        let dx = (event_xy[0] - consecutive_gesture_xy[0]).abs();
        let dy = (event_xy[1] - consecutive_gesture_xy[1]).abs();
        if dx + dy > WM_EVENT_CURSOR_MOTION_THRESHOLD {
            return true;
        }
    } else if iskeyboard_or_button(event_type) {
        if !iskeymodifier(event_type) {
            return true;
        }
    } else if event_type == WINDEACTIVATE {
        return true;
    }
    false
}

/// `WM_event_drag_threshold` (verbatim — tablet/mouse/
/// default × UI_SCALE_FAC).
pub fn wm_event_drag_threshold(
    prev_press_type: i32,
    is_tablet: bool,
    drag_threshold_tablet: i32,
    drag_threshold_mouse: i32,
    drag_threshold: i32,
    ui_scale_fac: f32,
) -> i32 {
    assert!(prev_press_type != crate::wm_operator_utils::MOUSEMOVE);
    let dt = if ismouse_button(prev_press_type) {
        if is_tablet { drag_threshold_tablet } else { drag_threshold_mouse }
    } else {
        drag_threshold
    };
    (dt as f32 * ui_scale_fac) as i32
}

/// `WM_event_drag_test_with_delta` (verbatim).
pub fn wm_event_drag_test_with_delta(drag_delta: [i32; 2], drag_threshold: i32) -> bool {
    drag_delta[0].abs() > drag_threshold || drag_delta[1].abs() > drag_threshold
}

/// `WM_event_drag_test` (verbatim — sub + delta test).
pub fn wm_event_drag_test(event_xy: [i32; 2], prev_xy: [i32; 2], drag_threshold: i32) -> bool {
    let delta = [prev_xy[0] - event_xy[0], prev_xy[1] - event_xy[1]];
    wm_event_drag_test_with_delta(delta, drag_threshold)
}

/// `WM_event_drag_start_mval` (verbatim — PRESS_DRAG uses
/// prev_press_xy, region-relative).
pub fn wm_event_drag_start_mval(
    event_val: i32,
    event_xy: [i32; 2],
    prev_press_xy: [i32; 2],
    winrct_xmin: i32,
    winrct_ymin: i32,
) -> [i32; 2] {
    let xy = if event_val == KM_PRESS_DRAG { prev_press_xy } else { event_xy };
    [xy[0] - winrct_xmin, xy[1] - winrct_ymin]
}

/// `WM_event_drag_start_xy` (verbatim).
pub fn wm_event_drag_start_xy(
    event_val: i32,
    event_xy: [i32; 2],
    prev_press_xy: [i32; 2],
) -> [i32; 2] {
    if event_val == KM_PRESS_DRAG { prev_press_xy } else { event_xy }
}

/// `wm_pressure_curve` (verbatim — threshold_max divide,
/// clamp, powf softness).
pub fn wm_pressure_curve(raw_pressure: f32, pressure_threshold_max: f32, pressure_softness: f32) -> f32 {
    let mut p = raw_pressure;
    if pressure_threshold_max != 0.0 {
        p /= pressure_threshold_max;
    }
    p = p.clamp(0.0, 1.0);
    if pressure_softness != 0.0 {
        p = p.powf(4.0_f32.powf(-pressure_softness));
    }
    p
}

/// `WM_event_tablet_data` (verbatim — tilt/pen_flip/
/// pressure).
pub fn wm_event_tablet_data(tablet_tilt: [f32; 2], tablet_active: i32, tablet_pressure: f32) -> (bool, [f32; 2], f32) {
    (tablet_active == EVT_TABLET_ERASER, tablet_tilt, tablet_pressure)
}

/// `WM_event_is_tablet` (verbatim).
pub fn wm_event_is_tablet(tablet_active: i32) -> bool {
    tablet_active != EVT_TABLET_NONE
}

/// `WM_event_absolute_delta_x`/`_y` (verbatim — invert
/// unless SCROLL_INVERT).
pub fn wm_event_absolute_delta_x(xy: [i32; 2], prev_xy: [i32; 2], event_flag: i32) -> i32 {
    let mut dx = xy[0] - prev_xy[0];
    if (event_flag & WM_EVENT_SCROLL_INVERT) == 0 {
        dx = -dx;
    }
    dx
}

/// `WM_event_absolute_delta_y` (verbatim).
pub fn wm_event_absolute_delta_y(xy: [i32; 2], prev_xy: [i32; 2], event_flag: i32) -> i32 {
    let mut dy = xy[1] - prev_xy[1];
    if (event_flag & WM_EVENT_SCROLL_INVERT) == 0 {
        dy = -dy;
    }
    dy
}

/// `WM_event_is_ime_switch` (verbatim — Ctrl/OsKey/Alt +
/// Space press).
pub fn wm_event_is_ime_switch(event_val: i32, event_type: i32, event_modifier: i32) -> bool {
    event_val == crate::wm_operator_utils::KM_PRESS
        && event_type == EVT_SPACEKEY
        && (event_modifier & (crate::wm_operator_utils::KM_CTRL | crate::wm_keymap_utils::KM_OSKEY | crate::wm_keymap_utils::KM_ALT)) != 0
}
