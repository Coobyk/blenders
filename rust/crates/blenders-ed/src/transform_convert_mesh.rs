/// Port of `editors/transform/transform_convert_mesh.cc` —
/// mesh editmode `createTransMesh_verts`:
/// per-selected-vert TransData with
/// loc=co, center=co.
use crate::transform::{TransData, TransInfo, TD_SELECTED};
/// `createTransMesh_verts` — TransData per
/// selected vert (index → co).
pub fn create_trans_mesh_verts(
    t: &mut TransInfo,
    verts: &[[f32; 3]],
    selected: &[bool],
) {
    for (i, co) in verts.iter().enumerate() {
        if !selected.get(i).copied().unwrap_or(false) {
            continue;
        }
        let mut td = TransData::default();
        td.identity();
        td.loc = *co;
        td.iloc = *co;
        td.center = *co;
        td.flag = TD_SELECTED;
        t.data.push(td);
    }
}
/// `createTransMeshEdges` — TransData per
/// selected edge (midpoint center).
pub fn create_trans_mesh_edges(
    t: &mut TransInfo,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    selected: &[bool],
) {
    for (i, e) in edges.iter().enumerate() {
        if !selected.get(i).copied().unwrap_or(false) {
            continue;
        }
        let a = verts[e[0] as usize];
        let b = verts[e[1] as usize];
        let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0, (a[2] + b[2]) / 2.0];
        let mut td = TransData::default();
        td.identity();
        td.loc = mid;
        td.iloc = mid;
        td.center = mid;
        td.flag = TD_SELECTED;
        t.data.push(td);
    }
}
