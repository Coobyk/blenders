/// Port of `editors/transform/transform_input.cc` —
/// `initMouseInput` + `applyMouseInput`
/// (numeric input modal handling).
use crate::transform::TransInfo;
/// `MouseInput` — parsed mouse delta.
#[derive(Clone, Copy, Default)]
pub struct MouseInput {
    /// `imval` — current [x,y] offset.
    pub imval: [f32; 2],
    /// `center` — screen center.
    pub center: [f32; 2],
    /// `precision_factor` (shift).
    pub precision_factor: f32,
    /// `virtual_mval`.
    pub virtual_mval: [f32; 2],
}
/// `inputNumberHorizontal`/`Vertical`
/// (verbatim — numpad digits appended).
pub fn input_number(
    current: f32,
    digit: i32,
    shift: bool,
) -> f32 {
    let d = digit as f32;
    if shift {
        // Decimal place.
        if current < 0.0 {
            current - d / 10.0
        } else {
            current + d / 10.0
        }
    } else {
        current * 10.0 + d
    }
}
/// `applyMouseInput` (verbatim — mouse delta
/// → `t->values` per mode precision).
pub fn apply_mouse_input(t: &mut TransInfo, mi: &MouseInput) {
    let dx = (mi.imval[0] - mi.center[0]) * mi.precision_factor;
    let dy = (mi.imval[1] - mi.center[1]) * mi.precision_factor;
    t.values[0] = dx;
    t.values[1] = dy;
    t.values_final[0] = t.values[0] + t.values_modal_offset[0];
    t.values_final[1] = t.values[1] + t.values_modal_offset[1];
}
/// `initMouseInput` — center + precision.
pub fn init_mouse_input(center: [f32; 2]) -> MouseInput {
    MouseInput {
        center,
        precision_factor: 1.0,
        ..Default::default()
    }
}
