/// Port of `extract_mesh_vbo_fdots_edituv_data.cc`
/// — fdot UV edit-flag VBO.
/// `extract_fdots_edituv_data` (verbatim —
/// face flag per fdot).
pub fn extract_fdots_edituv_data(
    mr: &super::extract_mesh::MeshRenderData,
    face_select: &[bool],
) -> Vec<u32> {
    (0..mr.faces.len())
        .map(|i| if face_select.get(i).copied().unwrap_or(false) { 1u32 } else { 0 })
        .collect()
}
