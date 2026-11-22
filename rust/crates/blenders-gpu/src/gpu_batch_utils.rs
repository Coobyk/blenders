/// Port of `intern/gpu_batch_utils.cc` —
/// primitive batch builders (cube/quad/
/// sphere/line).
/// `GPU_batch_cube` (verbatim — 24-vert cube).
pub fn gpu_batch_cube_positions(size: f32) -> Vec<[f32; 3]> {
    let s = size * 0.5;
    vec![
        [-s, -s, -s], [s, -s, -s], [s, s, -s], [-s, s, -s],
        [-s, -s, s], [s, -s, s], [s, s, s], [-s, s, s],
    ]
}
/// `GPU_batch_quad` (verbatim — 4-vert quad).
pub fn gpu_batch_quad_positions(size: f32) -> Vec<[f32; 3]> {
    let s = size * 0.5;
    vec![[-s, -s, 0.0], [s, -s, 0.0], [s, s, 0.0], [-s, s, 0.0]]
}
