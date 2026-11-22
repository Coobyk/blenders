/// Port of `editors/transform/transform_mode_tilt.cc` —
/// tilt modal: rotate curve points around
/// their tangent by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyTilt` (verbatim — per point:
/// tilt += value; write-back injected).
pub fn apply_tilt(t: &mut TransInfo, tilts: &mut [f32]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        if t.data[i].flag & TD_SKIP != 0 {
            continue;
        }
        if let Some(v) = tilts.get_mut(i) {
            *v += s;
        }
    }
}
/// `initTilt`.
pub fn init_tilt(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
