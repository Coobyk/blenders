/// Port of `extract_mesh_ibo_lines_paint_mask.cc`
/// — sculpt paint-mask edge IBO (only
/// edges with visible mask).
/// `extract_lines_paint_mask` (verbatim —
/// filter masked edges).
pub fn extract_lines_paint_mask(
    mr: &super::extract_mesh::MeshRenderData,
    mask: &[f32],
) -> Vec<u32> {
    let mut ibo = Vec::new();
    for &(v1, v2) in &mr.edges {
        let show = mask.get(v1 as usize).copied().unwrap_or(1.0) < 1.0
            || mask.get(v2 as usize).copied().unwrap_or(1.0) < 1.0;
        if show {
            ibo.extend([v1 as u32, v2 as u32]);
        }
    }
    ibo
}
