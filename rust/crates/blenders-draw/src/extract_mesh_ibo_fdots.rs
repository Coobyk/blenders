/// Port of `extract_mesh_ibo_fdots.cc` —
/// face-center point IBO (one point per
/// face).
/// `extract_fdots` (verbatim — idx per
/// face).
pub fn extract_fdots(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    (0..mr.faces.len() as u32).collect()
}
