/// Port of `extract_mesh_vbo_select_idx.cc`
/// — select-ID VBO (per-vert/edge/face
/// index for select buffer).
/// `extract_select_idx` (verbatim — id
/// start + idx).
pub fn extract_select_idx(start: u32, count: usize) -> Vec<u32> {
    (start..start + count as u32).collect()
}
