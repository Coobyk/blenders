/// Port of `extract_mesh_vbo_fdots_uv.cc` —
/// face-center UV VBO.
/// `extract_fdots_uv` (verbatim — avg uv).
pub fn extract_fdots_uv(
    mr: &super::extract_mesh::MeshRenderData,
    corner_uvs: &[[f32; 2]],
) -> Vec<[f32; 2]> {
    mr.faces
        .iter()
        .map(|&(start, size)| {
            let mut uv = [0.0f32; 2];
            for i in 0..size {
                uv[0] += corner_uvs[(start + i) as usize][0];
                uv[1] += corner_uvs[(start + i) as usize][1];
            }
            [uv[0] / size as f32, uv[1] / size as f32]
        })
        .collect()
}
