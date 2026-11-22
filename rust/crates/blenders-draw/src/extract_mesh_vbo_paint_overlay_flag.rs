/// Port of `extract_mesh_vbo_paint_overlay_flag.cc`
/// — paint overlay flag VBO (per-vert).
/// `extract_paint_overlay_flag` (verbatim
/// — flag bits).
pub fn extract_paint_overlay_flag(flags: &[u8]) -> Vec<u32> {
    flags.iter().map(|&f| f as u32).collect()
}
