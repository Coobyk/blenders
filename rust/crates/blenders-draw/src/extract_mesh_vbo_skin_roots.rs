/// Port of `extract_mesh_vbo_skin_roots.cc`
/// — skin-modifier root flag VBO.
/// `extract_skin_roots` (verbatim — flag
/// per vert).
pub fn extract_skin_roots(roots: &[u8]) -> Vec<u32> {
    roots.iter().map(|&r| r as u32).collect()
}
