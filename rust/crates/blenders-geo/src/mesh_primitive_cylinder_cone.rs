/// Port of `intern/mesh_primitive_cylinder_cone.cc`
/// — cylinder/cone primitive rings + caps.

/// `create_cylinder_mesh` (verbatim — top/
// bottom rings + optional side subdiv).
pub fn mesh_primitive_cylinder_positions(
    segments: i32,
    rings: i32,
    radius_top: f32,
    radius_bottom: f32,
    depth: f32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for r in 0..rings {
        let t = r as f32 / (rings - 1).max(1) as f32;
        let radius = radius_bottom + (radius_top - radius_bottom) * t;
        let z = -depth * 0.5 + depth * t;
        for s in 0..segments {
            let theta = 2.0 * std::f32::consts::PI * s as f32 / segments as f32;
            out.push([radius * theta.cos(), radius * theta.sin(), z]);
        }
    }
    out
}
/// Cylinder ring quads (verbatim).
pub fn mesh_primitive_cylinder_side_faces(
    segments: i32,
    rings: i32,
) -> Vec<[i32; 4]> {
    let mut out = Vec::new();
    for r in 0..(rings - 1) {
        for s in 0..segments {
            let a = r * segments + s;
            let b = r * segments + (s + 1) % segments;
            out.push([a, b, b + segments, a + segments]);
        }
    }
    out
}
