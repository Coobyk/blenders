/// Port of `extract_mesh_vbo_edge_idx.cc`
/// — per-edge index VBO.
/// `extract_edge_idx` (verbatim — idx per
/// edge).
pub fn extract_edge_idx(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    (0..mr.edges.len() as u32).collect()
}
