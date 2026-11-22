/// Port of `editors/transform/transform_mode_push_pull.cc` —
/// push/pull modal: move verts radially
/// from center by value.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyPushPull` (verbatim — loc = center
/// + normalize(iloc-center) * (dist + value)).
pub fn apply_push_pull(t: &mut TransInfo) {
    let s = t.values_final[0];
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
        let dist = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        let nd = (dist + s).max(0.0) / dist.max(1e-9);
        td.loc = [
            td.center[0] + d[0] * nd,
            td.center[1] + d[1] * nd,
            td.center[2] + d[2] * nd,
        ];
    }
}
/// `initPushPull`.
pub fn init_push_pull(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
