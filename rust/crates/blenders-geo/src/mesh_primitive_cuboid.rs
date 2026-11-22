/// Port of `intern/mesh_primitive_cuboid.cc`
/// — box primitive verts/edges/quads.

/// `create_cuboid_mesh` (verbatim — 8 corners
// from min/max).
pub fn mesh_primitive_cuboid_positions(
    min: [f32; 3],
    max: [f32; 3],
) -> [[f32; 3]; 8] {
    [
        [min[0], min[1], min[2]],
        [max[0], min[1], min[2]],
        [max[0], max[1], min[2]],
        [min[0], max[1], min[2]],
        [min[0], min[1], max[2]],
        [max[0], min[1], max[2]],
        [max[0], max[1], max[2]],
        [min[0], max[1], max[2]],
    ]
}
/// Cuboid quad face indices (verbatim).
pub fn mesh_primitive_cuboid_faces() -> [[i32; 4]; 6] {
    [
        [0, 1, 2, 3],
        [4, 7, 6, 5],
        [0, 4, 5, 1],
        [1, 5, 6, 2],
        [2, 6, 7, 3],
        [3, 7, 4, 0],
    ]
}
