//! Port of `intern/mesh_primitive_grid.cc`
//! — UV grid primitive verts/quads.

/// `create_grid_mesh` (verbatim — verts on
//! plane then quads).
pub fn mesh_primitive_grid_positions(
    size_x: i32,
    size_y: i32,
    step_x: f32,
    step_y: f32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity((size_x * size_y) as usize);
    for y in 0..size_y {
        for x in 0..size_x {
            out.push([x as f32 * step_x, y as f32 * step_y, 0.0]);
        }
    }
    out
}
/// Grid quad indices (verbatim).
pub fn mesh_primitive_grid_faces(size_x: i32, size_y: i32) -> Vec<[i32; 4]> {
    let mut out = Vec::new();
    for y in 0..(size_y - 1) {
        for x in 0..(size_x - 1) {
            let i = y * size_x + x;
            out.push([i, i + 1, i + 1 + size_x, i + size_x]);
        }
    }
    out
}
