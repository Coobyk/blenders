//! Port of `intern/join_geometries.cc` —
//! concat GeometrySets into one (offset
//! attribute domains).

/// `join_geometries` (verbatim — running
// offsets per component, concat).
pub fn join_geometries_offsets(element_counts: &[i32]) -> Vec<i32> {
    let mut out = Vec::with_capacity(element_counts.len());
    let mut offset = 0i32;
    for &n in element_counts {
        out.push(offset);
        offset += n;
    }
    out
}
