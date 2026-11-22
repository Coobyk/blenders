/// Port of `intern/mesh_primitive_uv_sphere.cc`
/// — UV sphere primitive verts/rings/quads.

/// `create_uv_sphere_mesh` (verbatim — ring
/// loops + poles).
pub fn mesh_primitive_uv_sphere_positions(
    segments: i32,
    rings: i32,
    radius: f32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    out.push([0.0, 0.0, radius]);
    for r in 1..rings {
        let phi = std::f32::consts::PI * r as f32 / rings as f32;
        let z = radius * phi.cos();
        let rho = radius * phi.sin();
        for s in 0..segments {
            let theta = 2.0 * std::f32::consts::PI * s as f32 / segments as f32;
            out.push([rho * theta.cos(), rho * theta.sin(), z]);
        }
    }
    out.push([0.0, 0.0, -radius]);
    out
}
/// UV sphere quad/tri indices (verbatim).
pub fn mesh_primitive_uv_sphere_faces(
    segments: i32,
    rings: i32,
) -> Vec<Vec<i32>> {
    let mut out = Vec::new();
    let bottom = 1 + (rings - 1) * segments;
    // Top pole fan.
    for s in 0..segments {
        out.push(vec![0, 1 + s, 1 + (s + 1) % segments]);
    }
    // Middle quads.
    for r in 0..(rings - 2) {
        for s in 0..segments {
            let a = 1 + r * segments + s;
            let b = 1 + r * segments + (s + 1) % segments;
            out.push(vec![a, b, b + segments, a + segments]);
        }
    }
    // Bottom pole fan.
    let last_ring = 1 + (rings - 2) * segments;
    for s in 0..segments {
        out.push(vec![
            bottom,
            last_ring + (s + 1) % segments,
            last_ring + s,
        ]);
    }
    out
}
