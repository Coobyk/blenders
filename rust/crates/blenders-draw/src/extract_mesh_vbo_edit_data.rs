/// Port of `extract_mesh_vbo_edit_data.cc`
/// — edit-mode flag VBO (vert/edge/face
/// select + crease).
/// `extract_edit_data` (verbatim — per-
/// corner EditLoopData).
pub fn extract_edit_data(
    mr: &super::extract_mesh::MeshRenderData,
    select: &[u32],
) -> Vec<super::extract_mesh::EditLoopData> {
    mr.corner_verts
        .iter()
        .map(|&v| super::extract_mesh::EditLoopData {
            v_flag: select.get(v as usize).copied().unwrap_or(0),
        })
        .collect()
}
