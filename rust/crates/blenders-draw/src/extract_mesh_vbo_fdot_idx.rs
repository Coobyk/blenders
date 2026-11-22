/// Port of `extract_mesh_vbo_fdot_idx.cc`
/// — per-face idx VBO.
/// `extract_fdot_idx` (verbatim — idx per
/// face).
pub fn extract_fdot_idx(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    (0..mr.faces.len() as u32).collect()
}
