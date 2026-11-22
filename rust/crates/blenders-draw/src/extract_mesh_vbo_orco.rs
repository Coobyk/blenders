/// Port of `extract_mesh_vbo_orco.cc` —
/// "Original Coordinates" (undeformed)
/// positions VBO.
/// `extract_orco` (verbatim — original
/// positions).
pub fn extract_orco(mr: &super::extract_mesh::MeshRenderData) -> Vec<[f32; 3]> {
    mr.vert_positions.clone()
}
