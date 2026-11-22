/// Port of `extract_mesh_ibo_points.cc` —
/// point IBO extract (all verts + loose
/// edge ends).
/// `extract_points` (verbatim — all verts,
/// then loose-edge endpoints).
pub fn extract_points(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    let mut ibo: Vec<u32> =
        (0..mr.vert_positions.len() as u32).collect();
    for &ei in &mr.loose_edges {
        let (v1, v2) = mr.edges[ei as usize];
        ibo.push(v1 as u32);
        ibo.push(v2 as u32);
    }
    ibo
}
