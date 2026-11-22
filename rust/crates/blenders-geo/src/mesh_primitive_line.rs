//! Port of `intern/mesh_primitive_line.cc`
//! — line mesh primitive (2 points 1 edge).

/// `create_line_mesh` (verbatim — endpoints,
// count, spacing).
pub fn mesh_primitive_line_positions(
    start: [f32; 3],
    delta: [f32; 3],
    count: i32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::with_capacity(count as usize);
    for i in 0..count {
        out.push([
            start[0] + delta[0] * i as f32,
            start[1] + delta[1] * i as f32,
            start[2] + delta[2] * i as f32,
        ]);
    }
    out
}
/// Line edges (verbatim — i..i+1 chain).
pub fn mesh_primitive_line_edges(count: i32) -> Vec<[i32; 2]> {
    (0..count.saturating_sub(1)).map(|i| [i, i + 1]).collect()
}
