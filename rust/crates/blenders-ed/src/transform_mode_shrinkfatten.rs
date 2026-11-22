/// Port of `editors/transform/transform_mode_shrinkfatten.cc` —
/// shrink/fatten modal: displace verts
/// along their normals by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyShrinkFatten` (verbatim — loc =
/// iloc + normal * (dist + value)).
pub fn apply_shrink_fatten(t: &mut TransInfo, normals: &[[f32; 3]]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        let n = normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
        td.loc = [
            td.iloc[0] + n[0] * s,
            td.iloc[1] + n[1] * s,
            td.iloc[2] + n[2] * s,
        ];
    }
}
/// `initShrinkFatten`.
pub fn init_shrink_fatten(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
