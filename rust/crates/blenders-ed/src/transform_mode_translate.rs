/// Port of `editors/transform/transform_mode_translate.cc` —
/// translate modal: mouse → world offset,
/// snapping + constraints applied.
use crate::transform::TransInfo;
/// `applyTranslation` input handling —
/// mouse dx/dy → `t->values` (projected
/// through view axis, injected).
pub fn translate_modal_input(
    t: &mut TransInfo,
    dx: f32,
    dy: f32,
    project: &mut dyn FnMut(f32, f32) -> [f32; 3],
) {
    let v = project(dx, dy);
    t.values[0] = v[0];
    t.values[1] = v[1];
    t.values[2] = v[2];
    t.values_final = [
        t.values[0] + t.values_modal_offset[0],
        t.values[1] + t.values_modal_offset[1],
        t.values[2] + t.values_modal_offset[2],
        0.0,
    ];
}
/// `initTranslation` — default mode +
/// center from cursor/median.
pub fn init_translation(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
    t.values_modal_offset = [0.0; 4];
}
