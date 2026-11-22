/// Port of `intern/mesh_merge_verts.cc` —
/// weld vertices by distance (sort + union).

/// `mesh_merge_verts` (verbatim — sorted
// spatial merge → merge-map).
pub fn mesh_merge_verts(
    positions: &[[f32; 3]],
    distance: f32,
) -> Vec<i32> {
    let n = positions.len();
    let mut merge_to: Vec<i32> = (0..n as i32).collect();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        positions[a]
            .partial_cmp(&positions[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let d2 = distance * distance;
    for i in 0..n {
        for j in (i + 1)..n {
            let a = positions[order[i]];
            let b = positions[order[j]];
            if (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2) > d2 {
                break;
            }
            merge_to[order[j]] = order[i] as i32;
        }
    }
    merge_to
}
