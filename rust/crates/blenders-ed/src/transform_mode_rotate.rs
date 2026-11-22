/// Port of `editors/transform/transform_mode_rotate.cc` —
/// rotate modal: mouse → angle (radians).
use crate::transform::TransInfo;
/// `applyRotation` input handling —
/// mouse delta → radians.
pub fn rotate_modal_input(t: &mut TransInfo, mouse_delta: f32) {
    // C: angle = delta * 0.01 (screen-space).
    let angle = mouse_delta * 0.01;
    t.values[0] = angle;
    t.values_final[0] = angle + t.values_modal_offset[0];
}
/// `initRotation`.
pub fn init_rotation(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
    t.values_modal_offset = [0.0; 4];
}
