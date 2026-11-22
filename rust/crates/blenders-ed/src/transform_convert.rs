/// Port of `transform/transform_convert.cc`
/// — `createTransData` dispatch: extract
/// `TransData` elements per object type.
/// `TC_*` container types (verbatim).
pub const TC_NONE: i32 = 0;
pub const TC_ACTION_DATA: i32 = 1;
pub const TC_GRAPH_EDIT_DATA: i32 = 2;
pub const TC_MESH_VERTS: i32 = 3;
pub const TC_MESH_EDGES: i32 = 4;
pub const TC_MESH_UV: i32 = 5;
pub const TC_OBJECT: i32 = 6;
pub const TC_CURSOR: i32 = 7;
/// `TransDataContainer` (verbatim —
/// one object/type's element array).
pub struct TransDataContainer {
    pub tc_type: i32,
    pub data: Vec<crate::transform::TransData>,
    /// `use_local_mat` (local-space xform).
    pub use_local_mat: bool,
    /// Local transform matrix.
    pub mat: [[f32; 4]; 4],
    /// Its inverse.
    pub imat: [[f32; 4]; 4],
}
/// `createTransObject` (verbatim — object
/// loc → TransData via inverse spacemtx).
pub fn create_trans_object(ob_loc: [f32; 3]) -> crate::transform::TransData {
    let mut td = crate::transform::TransData::default();
    td.iloc = ob_loc;
    td.loc = ob_loc;
    td.center = ob_loc;
    td.identity();
    td.flag = crate::transform::TD_SELECTED;
    td
}
/// `createTransMeshVerts` (verbatim —
/// verts → TransData).
pub fn create_trans_mesh_verts(
    verts: &[[f32; 3]],
    selected: &[bool],
) -> Vec<crate::transform::TransData> {
    let mut out = Vec::with_capacity(verts.len());
    for (i, v) in verts.iter().enumerate() {
        let mut td = crate::transform::TransData::default();
        td.iloc = *v;
        td.loc = *v;
        td.center = *v;
        td.identity();
        td.flag = if selected.get(i).copied().unwrap_or(false) {
            crate::transform::TD_SELECTED
        } else {
            0
        };
        out.push(td);
    }
    out
}
/// `applyTransObjects` (verbatim —
/// TransData.loc → object loc).
pub fn apply_trans_objects(
    tds: &[crate::transform::TransData],
    locs: &mut [[f32; 3]],
) {
    for (i, td) in tds.iter().enumerate() {
        if let Some(l) = locs.get_mut(i) {
            *l = td.loc;
        }
    }
}
