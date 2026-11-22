/// Port of `extract_mesh_vbo_fdots_nor.cc` —
/// face-normal VBO for fdots.
/// `extract_fdots_nor` (verbatim — Newell
/// face normal → packed).
pub fn extract_fdots_nor(
    mr: &super::extract_mesh::MeshRenderData,
) -> Vec<[i16; 4]> {
    mr.faces
        .iter()
        .map(|&(start, size)| {
            let mut n = [0.0f32; 3];
            let mut prev =
                mr.vert_positions[mr.corner_verts[(start + size - 1) as usize] as usize];
            for i in 0..size {
                let cur = mr.vert_positions
                    [mr.corner_verts[(start + i) as usize] as usize];
                n[0] += (prev[1] - cur[1]) * (prev[2] + cur[2]);
                n[1] += (prev[2] - cur[2]) * (prev[0] + cur[0]);
                n[2] += (prev[0] - cur[0]) * (prev[1] + cur[1]);
                prev = cur;
            }
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let n = if l > 0.0 { [n[0] / l, n[1] / l, n[2] / l] } else { n };
            [(n[0] * 32767.0) as i16, (n[1] * 32767.0) as i16, (n[2] * 32767.0) as i16, 0]
        })
        .collect()
}
