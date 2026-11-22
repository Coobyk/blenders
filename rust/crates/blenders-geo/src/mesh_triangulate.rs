/// Port of `intern/mesh_triangulate.cc` —
/// polygon → triangle indices (fan + CDT).

/// `mesh_triangulate` (verbatim — triangle fan
// for convex quads/ngons).
pub fn mesh_triangulate_offsets(
    poly_offsets: &[(i32, i32)],
) -> Vec<[i32; 3]> {
    let mut out = Vec::new();
    for &(s, e) in poly_offsets {
        let n = e - s;
        for i in 1..(n - 1) {
            out.push([s, s + i, s + i + 1]);
        }
    }
    out
}
