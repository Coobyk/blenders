/// Port of `extract_mesh_vbo_uv.cc` — UV
/// VBO extract (per-corner float2).
/// `extract_uvs` (verbatim — per-corner
/// uv layer gather).
pub fn extract_uvs(corner_uvs: &[[f32; 2]]) -> Vec<[f32; 2]> {
    corner_uvs.to_vec()
}
