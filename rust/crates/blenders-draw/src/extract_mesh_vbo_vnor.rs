/// Port of `extract_mesh_vbo_vnor.cc` —
/// vertex-normal VBO (packed short4).
/// `extract_normals` (verbatim — per-vert
/// normalize → pack).
pub fn extract_normals(normals: &[[f32; 3]]) -> Vec<[i16; 4]> {
    normals
        .iter()
        .map(|n| {
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let n = if l > 0.0 { [n[0] / l, n[1] / l, n[2] / l] } else { *n };
            [
                (n[0] * 32767.0) as i16,
                (n[1] * 32767.0) as i16,
                (n[2] * 32767.0) as i16,
                0,
            ]
        })
        .collect()
}
