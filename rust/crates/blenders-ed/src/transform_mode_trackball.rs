/// Port of `editors/transform/transform_mode_trackball.cc` —
/// trackball modal: 2-axis free rotation.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyTrackball` (verbatim — loc = center
/// + R(trackball)*(iloc-center)).
pub fn apply_trackball(t: &mut TransInfo) {
    let (phi, theta) = (t.values_final[0], t.values_final[1]);
    // Axis: x = [1,0,0], y = [0,1,0] combined.
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
        // Rotate about Y by phi then X by theta.
        let (sp, cp) = phi.sin_cos();
        let (st, ct) = theta.sin_cos();
        let r1 = [
            d[0] * cp + d[2] * sp,
            d[1],
            -d[0] * sp + d[2] * cp,
        ];
        let r2 = [
            r1[0],
            r1[1] * ct - r1[2] * st,
            r1[1] * st + r1[2] * ct,
        ];
        td.loc = [
            td.center[0] + r2[0],
            td.center[1] + r2[1],
            td.center[2] + r2[2],
        ];
    }
}
/// `initTrackball`.
pub fn init_trackball(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
