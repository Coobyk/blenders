/// Port of `editors/transform/transform_mode_resize.cc` —
/// resize modal: mouse → scale factor.
use crate::transform::TransInfo;
/// `applyResize` input handling —
/// mouse delta → ratio (px-scale).
pub fn resize_modal_input(t: &mut TransInfo, mouse_delta: f32) {
    // C: ratio = 1 + delta * 0.01 (screen-space).
    let ratio = 1.0 + mouse_delta * 0.01;
    t.values[0] = ratio;
    t.values_final = [
        ratio + t.values_modal_offset[0],
        ratio + t.values_modal_offset[1],
        ratio + t.values_modal_offset[2],
        0.0,
    ];
}
/// `initResize`.
pub fn init_resize(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [1.0, 1.0, 1.0, 0.0];
    t.values_final = [1.0, 1.0, 1.0, 0.0];
    t.values_modal_offset = [0.0; 4];
}
