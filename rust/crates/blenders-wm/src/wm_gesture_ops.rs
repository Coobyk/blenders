//! Port of `intern/wm_gesture_ops.cc` — generic gesture
//! ops (box/circle/lasso/lines/polyline/straightline):
//! modal-state→props, apply-rect, angle snap, OT reg.

/// `GESTURE_MODAL_*` verbatim.
pub const GESTURE_MODAL_SELECT: i32 = 1;
pub const GESTURE_MODAL_DESELECT: i32 = 2;
pub const GESTURE_MODAL_IN: i32 = 3;
pub const GESTURE_MODAL_OUT: i32 = 4;
/// `SEL_OP_*` verbatim.
pub const SEL_OP_ADD: i32 = 1;
pub const SEL_OP_SUB: i32 = 2;
/// `GESTURE_EDGE_PAN_*` verbatim.
pub const GESTURE_EDGE_PAN_INSIDE_PAD: f32 = 2.5;
pub const GESTURE_EDGE_PAN_OUTSIDE_PAD: f32 = 0.0;
pub const GESTURE_EDGE_PAN_SPEED_RAMP: f32 = 6.0;
pub const GESTURE_EDGE_PAN_MAX_SPEED: f32 = 10.0;
pub const GESTURE_EDGE_PAN_DELAY: f64 = 0.2;
pub const GESTURE_EDGE_PAN_ZOOM_INFLUENCE: f32 = 1.0;

/// `gesture_modal_state_to_operator` (verbatim —
/// deselect/mode/zoom_out).
pub enum GestureModalProp {
    /// `deselect = modal_state == DESELECT`.
    Deselect(bool),
    /// `mode = DESELECT ? SEL_OP_SUB : SEL_OP_ADD`.
    Mode(i32),
    /// `zoom_out = modal_state == OUT`.
    ZoomOut(bool),
}

/// Resolves `gesture_modal_state_to_operator` (verbatim).
pub fn gesture_modal_state_props(modal_state: i32, has_deselect: bool, has_mode: bool, has_zoom_out: bool) -> Vec<GestureModalProp> {
    let mut out = Vec::new();
    match modal_state {
        GESTURE_MODAL_SELECT | GESTURE_MODAL_DESELECT => {
            if has_deselect {
                out.push(GestureModalProp::Deselect(modal_state == GESTURE_MODAL_DESELECT));
            }
            if has_mode {
                out.push(GestureModalProp::Mode(
                    if modal_state == GESTURE_MODAL_DESELECT { SEL_OP_SUB } else { SEL_OP_ADD },
                ));
            }
        }
        GESTURE_MODAL_IN | GESTURE_MODAL_OUT => {
            if has_zoom_out {
                out.push(GestureModalProp::ZoomOut(modal_state == GESTURE_MODAL_OUT));
            }
        }
        _ => {}
    }
    out
}

/// `gesture_box_apply_rect` (verbatim — empty-rect gate +
/// normalized min/max).
pub fn gesture_box_apply_rect(rect: &crate::wm_subwindow::Rcti) -> Option<crate::wm_subwindow::Rcti> {
    if rect.xmin == rect.xmax || rect.ymin == rect.ymax {
        return None;
    }
    Some(crate::wm_subwindow::Rcti {
        xmin: rect.xmin.min(rect.xmax),
        ymin: rect.ymin.min(rect.ymax),
        xmax: rect.xmin.max(rect.xmax),
        ymax: rect.ymin.max(rect.ymax),
    })
}

/// `WM_gesture_box_invoke` (verbatim — cross-rect when
/// wait_for_input, else rect).
pub fn gesture_box_invoke_type(event_is_mouse_drag_or_press: bool, wait_for_input_prop: bool) -> i32 {
    if !event_is_mouse_drag_or_press && wait_for_input_prop {
        crate::wm_gesture::WM_GESTURE_CROSS_RECT
    } else {
        crate::wm_gesture::WM_GESTURE_RECT
    }
}

/// `gesture_polyline_valid_points` (verbatim — last point
/// != mval → +1).
pub fn gesture_polyline_valid_points(points: &[(i16, i16)], mval: (i16, i16), is_click_submitted: bool) -> usize {
    let num_points = points.len();
    if is_click_submitted {
        return num_points;
    }
    if num_points == 0 {
        return 0;
    }
    let (px, py) = points[num_points - 1];
    if mval.0 == px && mval.1 == py {
        num_points
    } else {
        num_points + 1
    }
}

/// `gesture_polyline_can_apply` (verbatim — ≥2 submitted +
/// >2 valid).
pub fn gesture_polyline_can_apply(num_points: usize, valid_points: usize) -> bool {
    if num_points < 2 {
        return false;
    }
    valid_points > 2
}

/// `wm_gesture_straightline_do_angle_snap` (verbatim —
/// snap direction to `snap_angle`, then 45° |dx|=|dy|).
pub fn wm_gesture_straightline_do_angle_snap(rect: &mut crate::wm_subwindow::Rcti, snap_angle: f32) {
    let line_start = [rect.xmin as f32, rect.ymin as f32];
    let line_end = [rect.xmax as f32, rect.ymax as f32];
    let x_axis = [1.0f32, 0.0];
    let mut dir = [line_end[0] - line_start[0], line_end[1] - line_start[1]];
    let len = (dir[0] * dir[0] + dir[1] * dir[1]).sqrt();
    if len != 0.0 {
        dir[0] /= len;
        dir[1] /= len;
    }
    /* `angle_signed_v2v2(x_axis, dir) = atan2(cross, dot)`. */
    let cross = x_axis[0] * dir[1] - x_axis[1] * dir[0];
    let dot = x_axis[0] * dir[0] + x_axis[1] * dir[1];
    let cur = cross.atan2(dot);
    let adjusted = cur + snap_angle / 2.0;
    let angle_snapped = -(adjusted / snap_angle).floor() * snap_angle;
    let (s, c) = angle_snapped.sin_cos();
    let end = [line_start[0] + c * len, line_start[1] + s * len];
    rect.xmax = end[0] as i32;
    rect.ymax = end[1] as i32;
    let deg45 = 45.0f32.to_radians();
    let deg90 = 90.0f32.to_radians();
    let fract45 = angle_snapped / deg45 - (angle_snapped / deg45).floor();
    let fract90 = angle_snapped / deg90 - (angle_snapped / deg90).floor();
    let near = |v: f32| (v.abs() < 1e-6) || ((v.abs() - 1.0).abs() < 1e-6);
    if near(fract45) && !near(fract90) {
        let xlen = (rect.xmax - rect.xmin).abs();
        let mut ylen = rect.ymax - rect.ymin;
        if ylen.abs() != xlen {
            ylen = xlen * if ylen >= 0 { 1 } else { -1 };
            rect.ymax = rect.ymin + ylen;
        }
    }
}

/// OT registrations (verbatim strings).
pub fn wm_ot_circle_gesture() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_circle_gesture", "Circle Gesture", "Draw a circle defined by the cursor")
}
pub fn wm_ot_lasso_gesture() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_lasso_gesture", "Lasso Gesture", "Draw a lasso defined by the cursor")
}
pub fn wm_ot_polyline_gesture() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_polyline_gesture", "Polyline Gesture", "Draw a polyline defined by the cursor")
}
pub fn wm_ot_straightline_gesture() -> (&'static str, &'static str, &'static str) {
    ("WM_OT_straightline_gesture", "Straight Line Gesture", "Draw a straight line defined by the cursor")
}
