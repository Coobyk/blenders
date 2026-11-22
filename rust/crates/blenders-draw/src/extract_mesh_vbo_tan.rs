/// Port of `extract_mesh_vbo_tan.cc` —
/// tangent VBO (packed short4, w=handedness).
/// `extract_tan` (verbatim — per-corner
/// tangent).
pub fn extract_tan(tangents: &[[f32; 4]]) -> Vec<[i16; 4]> {
    tangents
        .iter()
        .map(|t| {
            let l = (t[0] * t[0] + t[1] * t[1] + t[2] * t[2]).sqrt();
            let t = if l > 0.0 { [t[0] / l, t[1] / l, t[2] / l, t[3]] } else { *t };
            [
                (t[0] * 32767.0) as i16,
                (t[1] * 32767.0) as i16,
                (t[2] * 32767.0) as i16,
                t[3].signum() as i16 * 32767,
            ]
        })
        .collect()
}
