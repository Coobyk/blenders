//! Port of `intern/separate_geometry.cc` —
//! split selection into a new GeometrySet.

/// `separate_geometry` (verbatim — per-domain
//! partition by selection mask).
pub fn separate_geometry_indices(
    selection: &[bool],
) -> (Vec<i32>, Vec<i32>) {
    let mut keep = Vec::new();
    let mut moved = Vec::new();
    for (i, &s) in selection.iter().enumerate() {
        if s {
            moved.push(i as i32);
        } else {
            keep.push(i as i32);
        }
    }
    (keep, moved)
}
