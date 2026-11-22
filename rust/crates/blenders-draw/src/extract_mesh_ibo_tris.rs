/// Port of `extract_mesh_ibo_tris.cc` —
/// triangle IBO extract (corner tris +
/// loose verts as degenerates).
/// `extract_tris` (verbatim — append loose
/// verts).
pub fn extract_tris(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    let nverts = mr.vert_positions.len() as i32;
    let mut ibo: Vec<u32> = mr
        .loop_tris
        .iter()
        .flat_map(|t| t.iter().map(|&c| c as u32))
        .collect();
    // Loose verts → degenerate tris (v,v,v).
    for &vi in &mr.loose_verts {
        let v = (nverts - mr.loose_verts.len() as i32 + vi) as u32;
        ibo.extend([v, v, v]);
    }
    ibo
}
