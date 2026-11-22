/// Port of `editors/transform/transform_mode_track.cc` —
/// track modal: rotate verts to align
/// a normal with a target.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyTrack` (verbatim — trackball
/// quaternion about view axis → rotate).
pub fn apply_track(t: &mut TransInfo, normal: [f32; 3]) {
    let angle = t.values_final[0];
    let (s, c) = angle.sin_cos();
    let oc = 1.0 - c;
    let (x, y, z) = (normal[0], normal[1], normal[2]);
    let m = [
        [c + x * x * oc, x * y * oc - z * s, x * z * oc + y * s],
        [y * x * oc + z * s, c + y * y * oc, y * z * oc - x * s],
        [z * x * oc - y * s, z * y * oc + x * s, c + z * z * oc],
    ];
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        let d = [
            td.iloc[0] - td.center[0],
            td.iloc[1] - td.center[1],
            td.iloc[2] - td.center[2],
        ];
        td.loc = [
            td.center[0] + m[0][0] * d[0] + m[0][1] * d[1] + m[0][2] * d[2],
            td.center[1] + m[1][0] * d[0] + m[1][1] * d[1] + m[1][2] * d[2],
            td.center[2] + m[2][0] * d[0] + m[2][1] * d[1] + m[2][2] * d[2],
        ];
    }
}
/// `initTrack`.
pub fn init_track(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
