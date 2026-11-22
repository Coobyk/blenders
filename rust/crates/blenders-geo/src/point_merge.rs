//! Port of `intern/point_merge.cc` — merge
//! points by distance (ver_sorted dedup).

/// `merge_points` (verbatim — sort by pos,
// union neighbors within radius).
pub fn point_merge_indices(
    positions: &[[f32; 3]],
    radius: f32,
) -> Vec<i32> {
    let n = positions.len();
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        positions[a]
            .partial_cmp(&positions[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut merge_to: Vec<i32> = (0..n as i32).collect();
    let r2 = radius * radius;
    for i in 0..n {
        for j in (i + 1)..n {
            let a = positions[order[i]];
            let b = positions[order[j]];
            let d2 = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
            if d2 > r2 {
                break;
            }
            merge_to[order[j]] = order[i] as i32;
        }
    }
    merge_to
}
