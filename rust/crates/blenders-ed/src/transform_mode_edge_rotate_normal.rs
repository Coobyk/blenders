/// Port of `editors/transform/transform_mode_edge_rotate_normal.cc` —
/// rotate-around-normal modal.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyEdgeRotateNormal` (verbatim —
/// loc = center + R(normal, angle)·d).
pub fn apply_edge_rotate_normal(t: &mut TransInfo, normals: &[[f32; 3]]) {
    let angle = t.values_final[0];
    let (s, c) = angle.sin_cos();
    let oc = 1.0 - c;
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        let a = normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
        let d = [
            td.iloc[0] - td.center[0],
            td.iloc[1] - td.center[1],
            td.iloc[2] - td.center[2],
        ];
        let (x, y, z) = (a[0], a[1], a[2]);
        let m = [
            [c + x * x * oc, x * y * oc - z * s, x * z * oc + y * s],
            [y * x * oc + z * s, c + y * y * oc, y * z * oc - x * s],
            [z * x * oc - y * s, z * y * oc + x * s, c + z * z * oc],
        ];
        td.loc = [
            td.center[0] + m[0][0] * d[0] + m[0][1] * d[1] + m[0][2] * d[2],
            td.center[1] + m[1][0] * d[0] + m[1][1] * d[1] + m[1][2] * d[2],
            td.center[2] + m[2][0] * d[0] + m[2][1] * d[1] + m[2][2] * d[2],
        ];
    }
}
/// `initEdgeRotateNormal`.
pub fn init_edge_rotate_normal(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
