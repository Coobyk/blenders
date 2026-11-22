/// Port of `extract_mesh_vbo_vert_idx.cc`
/// — per-vert index VBO.
/// `extract_vert_idx` (verbatim — idx per
/// vert).
pub fn extract_vert_idx(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    (0..mr.vert_positions.len() as u32).collect()
}
