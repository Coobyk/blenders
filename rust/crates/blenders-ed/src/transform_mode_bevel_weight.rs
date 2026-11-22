/// Port of `editors/transform/transform_mode_bevel_weight.cc` —
/// bevel-weight modal: adjust edge
/// bevel weights by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyBevelWeight` (verbatim — weight
/// += value, clamped [0,1]).
pub fn apply_bevel_weight(t: &mut TransInfo, weights: &mut [f32]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        if t.data[i].flag & TD_SKIP != 0 {
            continue;
        }
        if let Some(w) = weights.get_mut(i) {
            *w = (*w + s).clamp(0.0, 1.0);
        }
    }
}
/// `initBevelWeight`.
pub fn init_bevel_weight(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
