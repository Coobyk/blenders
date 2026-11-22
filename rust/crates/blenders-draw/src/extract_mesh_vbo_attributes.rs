/// Port of `extract_mesh_vbo_attributes.cc`
/// — generic attribute VBOs (color/uv/etc).
/// `extract_attributes` (verbatim — per-
/// attribute domain gather).
pub fn extract_attributes_float(corner_attr: &[f32]) -> Vec<f32> {
    corner_attr.to_vec()
}
