/// Port of `extract_mesh_vbo_lnor.cc` —
/// loop (corner) normal VBO.
/// `extract_lnor` (verbatim — packed
/// short4 per corner).
pub fn extract_lnor(corner_normals: &[[f32; 3]]) -> Vec<[i16; 4]> {
    super::extract_mesh_vbo_vnor::extract_normals(corner_normals)
}
