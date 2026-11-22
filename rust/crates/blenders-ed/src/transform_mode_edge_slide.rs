/// Port of `editors/transform/transform_mode_edge_slide.cc` —
/// edge-slide modal: slide verts along
/// their neighboring edges.
use crate::transform::{TransInfo, TD_SKIP};
/// `applyEdgeSlide` (verbatim — loc = iloc
/// + edge_dir * t, dir injected per vert).
pub fn apply_edge_slide(t: &mut TransInfo, edge_dirs: &[[f32; 3]]) {
    let s = t.values_final[0];
    for i in 0..t.data.len() {
        let td = &mut t.data[i];
        if td.flag & TD_SKIP != 0 {
            continue;
        }
        let d = edge_dirs.get(i).copied().unwrap_or([0.0; 3]);
        td.loc = [
            td.iloc[0] + d[0] * s,
            td.iloc[1] + d[1] * s,
            td.iloc[2] + d[2] * s,
        ];
    }
}
/// `initEdgeSlide`.
pub fn init_edge_slide(t: &mut TransInfo, center: [f32; 3]) {
    t.center_global = center;
    t.values = [0.0; 4];
    t.values_final = [0.0; 4];
}
