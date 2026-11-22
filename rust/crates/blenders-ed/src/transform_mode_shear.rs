/// Port of `editors/transform/transform_mode_shear.cc` —
/// shear modal: apply value as shear on
/// a 2D plane.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyShear` (verbatim — loc = iloc +
/// mat·(iloc-center), mat from shear).
pub fn apply_shear(t: &mut TransInfo, axis_h: [f32; 3], axis_v: [f32; 3]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        // Shear: displace along axis_h by s * proj(axis_v).
        let d = [
            td.iloc[0] - td.center[0],
            td.iloc[1] - td.center[1],
            td.iloc[2] - td.center[2],
        ];
        let proj = d[0] * axis_v[0] + d[1] * axis_v[1] + d[2] * axis_v[2];
        td.loc = [
            td.iloc[0] + axis_h[0] * proj * s,
            td.iloc[1] + axis_h[1] * proj * s,
            td.iloc[2] + axis_h[2] * proj * s,
        ];
    }
}
/// `initShear` — default center.
pub fn init_shear(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
