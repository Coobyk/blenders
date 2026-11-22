/// Port of `extract_mesh_vbo_edge_fac.cc` —
/// edge-factor VBO (crease/bevel-weight
/// display).
/// `extract_edge_fac` (verbatim — 0-255).
pub fn extract_edge_fac(facs: &[f32]) -> Vec<u8> {
    facs.iter().map(|f| (f.clamp(0.0, 1.0) * 255.0) as u8).collect()
}
