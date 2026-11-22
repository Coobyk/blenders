/// Port of `extract_mesh_vbo_weights.cc` —
/// vertex-group weight VBO.
/// `extract_weights` (verbatim — active
/// defgroup per-vert weight, packed 0-255).
pub fn extract_weights(weights: &[f32]) -> Vec<u8> {
    weights
        .iter()
        .map(|w| (w.clamp(0.0, 1.0) * 255.0) as u8)
        .collect()
}
