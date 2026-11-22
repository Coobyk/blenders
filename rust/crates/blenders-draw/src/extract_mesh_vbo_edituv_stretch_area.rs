/// Port of `extract_mesh_vbo_edituv_stretch_area.cc`
/// — UV stretch-area VBO (per-corner area
/// ratio).
/// `extract_edituv_stretch_area` (verbatim
/// — 3d/uv area ratio).
pub fn extract_edituv_stretch_area(
    area_3d: &[f32],
    area_uv: &[f32],
) -> Vec<f32> {
    area_3d
        .iter()
        .zip(area_uv)
        .map(|(a3, a2)| if *a3 > 0.0 { a2 / a3 } else { 0.0 })
        .collect()
}
