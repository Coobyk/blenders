/// Port of `extract_mesh_vbo_pos.cc` —
/// positions VBO extract (+ loose data).
/// `extract_vert_positions` (verbatim —
/// copy positions + loose edge/vert data).
pub fn extract_vert_positions(
    mr: &super::extract_mesh::MeshRenderData,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let vert_data = mr.vert_positions.clone();
    // Loose data: loose edges (v1,v2 each) + loose verts appended.
    let mut loose_data =
        Vec::with_capacity(mr.loose_edges.len() * 2 + mr.loose_verts.len());
    for &ei in &mr.loose_edges {
        let (v1, v2) = mr.edges[ei as usize];
        loose_data.push(mr.vert_positions[v1 as usize]);
        loose_data.push(mr.vert_positions[v2 as usize]);
    }
    for &vi in &mr.loose_verts {
        loose_data.push(mr.vert_positions[vi as usize]);
    }
    (vert_data, loose_data)
}
