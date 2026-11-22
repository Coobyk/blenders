/// Port of `extract_mesh_ibo_lines.cc` —
/// line IBO extract (edges + loose-point
/// degenerates).
/// `extract_lines` (verbatim — edges plus
/// (v,v) for loose verts).
pub fn extract_lines(mr: &super::extract_mesh::MeshRenderData) -> Vec<u32> {
    let mut ibo: Vec<u32> = mr
        .edges
        .iter()
        .flat_map(|&(v1, v2)| [v1 as u32, v2 as u32])
        .collect();
    for &vi in &mr.loose_verts {
        let v = vi as u32;
        ibo.extend([v, v]);
    }
    ibo
}
