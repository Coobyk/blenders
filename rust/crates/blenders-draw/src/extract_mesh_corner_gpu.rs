/// Port of `extract_mesh_corner_gpu.cc` —
/// GPU-side corner→vert expand.
/// `extract_corner_gpu` (verbatim — gather
/// corner verts).
pub fn extract_corner_gpu(corner_verts: &[i32]) -> Vec<u32> {
    corner_verts.iter().map(|&v| v as u32).collect()
}
