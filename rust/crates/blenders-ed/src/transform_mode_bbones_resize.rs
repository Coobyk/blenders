/// Port of `editors/transform/transform_mode_bbones_resize.cc` —
/// bbones-resize modal: scale bbone
/// segments by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyBonesSize` (verbatim — per bone:
/// scale x/y/z by value).
pub fn apply_bbones_resize(t: &mut TransInfo, scales: &mut [[f32; 3]]) {
    let s = [
        t.values_final[0],
        t.values_final[1],
        t.values_final[2],
    ];
    for i in 0..t.data.len() {
        if t.data[i].flag & TD_SKIP != 0 {
            continue;
        }
        if let Some(sc) = scales.get_mut(i) {
            sc[0] *= s[0];
            sc[1] *= s[1];
            sc[2] *= s[2];
        }
    }
}
/// `initBoneSize`.
pub fn init_bbones_resize(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [1.0, 1.0, 1.0, 0.0];
    t.values_final = [1.0, 1.0, 1.0, 0.0];
}
