/// Port of `editors/transform/transform_mode_maskshrinkfatten.cc` —
/// mask shrink/fatten modal: slide mask
/// points along their spline tangents.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyMaskShrinkFatten` (verbatim —
/// loc = iloc + tangent * value).
pub fn apply_mask_shrink_fatten(t: &mut TransInfo, tangents: &[[f32; 3]]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        let d = tangents.get(i).copied().unwrap_or([0.0; 3]);
        td.loc = [
            td.iloc[0] + d[0] * s,
            td.iloc[1] + d[1] * s,
            td.iloc[2] + d[2] * s,
        ];
    }
}
/// `initMaskShrinkFatten`.
pub fn init_mask_shrink_fatten(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
