/// Port of `extract_mesh_vbo_fdots_pos.cc` —
/// face-center position VBO.
/// `extract_fdots_pos` (verbatim — face
/// center average).
pub fn extract_fdots_pos(
    mr: &super::extract_mesh::MeshRenderData,
) -> Vec<[f32; 3]> {
    mr.faces
        .iter()
        .map(|&(start, size)| {
            let mut c = [0.0f32; 3];
            for i in 0..size {
                let v = mr.vert_positions
                    [mr.corner_verts[(start + i) as usize] as usize];
                for k in 0..3 {
                    c[k] += v[k];
                }
            }
            for k in 0..3 {
                c[k] /= size as f32;
            }
            c
        })
        .collect()
}
