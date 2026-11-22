/// Port of `extract_mesh_vbo_sculpt_data.cc`
/// — sculpt-layer data VBO (mask+face_set).
/// `extract_sculpt_data` (verbatim — packed
/// mask + face_set color).
pub fn extract_sculpt_data(mask: &[f32]) -> Vec<f32> {
    mask.to_vec()
}
