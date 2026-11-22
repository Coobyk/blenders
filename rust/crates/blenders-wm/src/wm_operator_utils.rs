//! Port of `intern/wm_operator_utils.cc` —
//! `WM_operator_flag_only_pass_through_on_press` +
//! interactive-value modal framework
//! (`ValueInteraction`, `op_generic_value_*`).

/// `KM_PRESS`/`KM_RELEASE` verbatim (`WM_types.h`).
pub const KM_PRESS: i32 = 1;
pub const KM_RELEASE: i32 = 0;
/// `OPERATOR_PASS_THROUGH`/`RUNNING_MODAL` verbatim.
pub const OPERATOR_PASS_THROUGH: i32 = 4;
pub const OPERATOR_RUNNING_MODAL: i32 = 8;
/// Event types used.
pub const MOUSEMOVE: i32 = 0x0001;
pub const EVT_LEFTCTRLKEY: i32 = 0x00D4;
pub const EVT_RIGHTCTRLKEY: i32 = 0x00D3;
pub const EVT_LEFTSHIFTKEY: i32 = 0x00D2;
pub const EVT_RIGHTSHIFTKEY: i32 = 0x00D1;
pub const EVT_RETKEY: i32 = 0x00B2;
pub const EVT_PADENTER: i32 = 0x0095;
pub const EVT_ESCKEY: i32 = 0x00B1;
pub const LEFTMOUSE: i32 = 0x0002;
pub const RIGHTMOUSE: i32 = 0x0004;
/// `KM_CTRL`/`KM_SHIFT` verbatim.
pub const KM_CTRL: i32 = 2;
pub const KM_SHIFT: i32 = 1;
/// `G_TRANSFORM_EDIT` verbatim.
pub const G_TRANSFORM_EDIT: i32 = 1 << 0;

/// `WM_operator_flag_only_pass_through_on_press`
/// (verbatim — strip PASS_THROUGH on finish/cancel for
/// non-press events).
pub fn wm_operator_flag_only_pass_through_on_press(retval: i32, event_val: i32) -> i32 {
    let mut retval = retval;
    if event_val != KM_PRESS {
        if (retval & OPERATOR_PASS_THROUGH) != 0 {
            assert!((retval & !OPERATOR_PASS_THROUGH) != 0);
            if (retval & (crate::wm_files_colorspace::OPERATOR_FINISHED | crate::wm_files_colorspace::OPERATOR_CANCELLED)) != 0 {
                retval &= !OPERATOR_PASS_THROUGH;
            }
        }
    }
    retval
}

/// `ValueInteraction` verbatim.
pub struct ValueInteraction {
    /// `init.mval`.
    pub init_mval: [f32; 2],
    /// `init.prop_value`.
    pub init_prop_value: f32,
    /// `prev.prop_value`.
    pub prev_prop_value: f32,
    /// `prev.is_snap`/`is_precise`.
    pub prev_is_snap: bool,
    pub prev_is_precise: bool,
    /// `range`.
    pub range: [f32; 2],
    /// `context_vars.area`/`region` + winx.
    pub context_area: usize,
    pub context_region_winx: i32,
}

/// `interactive_value_init` (verbatim).
pub fn interactive_value_init(
    inter: &mut ValueInteraction,
    event_mval: [f32; 2],
    value_final: f32,
    range: [f32; 2],
    area: usize,
    region_winx: i32,
) {
    inter.context_area = area;
    inter.context_region_winx = region_winx;
    inter.init_mval = event_mval;
    inter.init_prop_value = value_final;
    inter.prev_prop_value = value_final;
    inter.range = range;
}

/// `interactive_value_update` (verbatim — mval[0] scaled
/// delta, ctrl snap 0.1, shift ×0.1, status text on
/// change).
pub fn interactive_value_update(
    inter: &mut ValueInteraction,
    event_mval_0: i32,
    event_modifier: i32,
    area_status_text: &mut dyn FnMut(usize, Option<String>),
    r_value_final: &mut f32,
) -> bool {
    let mval_axis = 0;
    let value_scale = 4.0f32;
    let value_range = inter.range[1] - inter.range[0];
    let mval_curr = event_mval_0;
    let mval_init = inter.init_mval[mval_axis] as i32;
    let mut value_delta = (inter.init_prop_value
        + ((mval_curr - mval_init) as f32 / inter.context_region_winx as f32) * value_range)
        * value_scale;
    if (event_modifier & KM_CTRL) != 0 {
        let snap = 0.1f64;
        value_delta = ((value_delta as f64) / snap).round() as f32 * snap as f32;
    }
    if (event_modifier & KM_SHIFT) != 0 {
        value_delta *= 0.1;
    }
    let value_final = inter.init_prop_value + value_delta;
    let changed = value_final != inter.prev_prop_value;
    if changed {
        (area_status_text)(inter.context_area, Some(format!("{:.4}", value_final)));
    }
    inter.prev_prop_value = value_final;
    inter.prev_is_snap = (event_modifier & KM_CTRL) != 0;
    inter.prev_is_precise = (event_modifier & KM_SHIFT) != 0;
    *r_value_final = value_final;
    changed
}

/// `interactive_value_exit` (verbatim — clear status
/// text).
pub fn interactive_value_exit(
    inter: &mut ValueInteraction,
    area_status_text: &mut dyn FnMut(usize, Option<String>),
) {
    (area_status_text)(inter.context_area, None);
}

/// `ObCustomData_ForEditMode` verbatim.
pub struct ObCustomDataForEditMode {
    /// `launch_event`.
    pub launch_event: i32,
    /// `wait_for_input`.
    pub wait_for_input: bool,
    /// `is_active`.
    pub is_active: bool,
    /// `is_first`.
    pub is_first: bool,
    /// `inter`.
    pub inter: ValueInteraction,
    /// `objects_xform` (opaque handles).
    pub objects_xform: Vec<usize>,
}

/// `op_generic_value_modal` (verbatim — launch-release →
/// finish; move/ctrl/shift → update+exec; confirm/cancel
/// keys).
#[allow(clippy::too_many_arguments)]
pub fn op_generic_value_modal(
    cd: &mut ObCustomDataForEditMode,
    event_type: i32,
    event_val: i32,
    event_mval_0: i32,
    event_modifier: i32,
    area_status_text: &mut dyn FnMut(usize, Option<String>),
    prop_float_set: &mut dyn FnMut(f32),
    restore: &mut dyn FnMut(&mut ObCustomDataForEditMode),
    exit: &mut dyn FnMut(&mut ObCustomDataForEditMode),
    exec: &mut dyn FnMut() -> i32,
) -> i32 {
    if event_type == cd.launch_event && event_val == KM_RELEASE {
        if !cd.wait_for_input {
            (exit)(cd);
            return crate::wm_files_colorspace::OPERATOR_FINISHED;
        }
    }
    match event_type {
        t if t == MOUSEMOVE
            || t == EVT_LEFTCTRLKEY
            || t == EVT_RIGHTCTRLKEY
            || t == EVT_LEFTSHIFTKEY
            || t == EVT_RIGHTSHIFTKEY =>
        {
            let mut value_final = 0.0f32;
            if cd.is_active
                && interactive_value_update(
                    &mut cd.inter,
                    event_mval_0,
                    event_modifier,
                    area_status_text,
                    &mut value_final,
                )
            {
                (prop_float_set)(value_final);
                if !cd.is_first {
                    (restore)(cd);
                }
                let retval = (exec)();
                cd.is_first = false;
                if (retval & crate::wm_files_colorspace::OPERATOR_FINISHED) == 0 {
                    (exit)(cd);
                    return crate::wm_files_colorspace::OPERATOR_CANCELLED;
                }
            }
        }
        t if t == EVT_RETKEY || t == EVT_PADENTER || t == LEFTMOUSE => {
            if cd.wait_for_input {
                if event_val == KM_PRESS {
                    if !cd.is_active {
                        cd.is_active = true;
                    }
                } else if event_val == KM_RELEASE && cd.is_active {
                    (exit)(cd);
                    return crate::wm_files_colorspace::OPERATOR_FINISHED;
                }
            } else if event_val == KM_RELEASE {
                (exit)(cd);
                return crate::wm_files_colorspace::OPERATOR_FINISHED;
            }
        }
        t if t == EVT_ESCKEY || t == RIGHTMOUSE => {
            if event_val == KM_PRESS {
                if cd.is_active {
                    (restore)(cd);
                }
                (exit)(cd);
                return crate::wm_files_colorspace::OPERATOR_CANCELLED;
            }
        }
        _ => {}
    }
    OPERATOR_RUNNING_MODAL
}
