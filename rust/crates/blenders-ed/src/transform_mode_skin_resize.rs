/// Port of `editors/transform/transform_mode_skin_resize.cc` —
/// skin-resize modal: adjust skin-node
/// radii by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applySkinResize` (verbatim — radius
/// += value, clamped > 0).
pub fn apply_skin_resize(t: &mut TransInfo, radii: &mut [[f32; 2]]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        if t.data[i].flag & TD_SKIP != 0 {
            continue;
        }
        if let Some(r) = radii.get_mut(i) {
            r[0] = (r[0] + s).max(0.0);
            r[1] = (r[1] + s).max(0.0);
        }
    }
}
/// `initSkinResize`.
pub fn init_skin_resize(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
