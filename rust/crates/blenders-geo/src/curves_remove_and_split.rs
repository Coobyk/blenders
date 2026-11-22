//! Port of `intern/curves_remove_and_split.cc`
//! — remove points / split curves at point
//! selections, recomputing offsets.

/// `remove_points_and_split` (verbatim —
//! rebuild offsets around removed points).
pub fn curves_remove_and_split_offsets(
    points_by_curve: &[(i32, i32)],
    point_deleted: &[bool],
) -> Vec<(i32, i32)> {
    let mut out: Vec<(i32, i32)> = Vec::new();
    let mut write = 0i32;
    for &(start, end) in points_by_curve {
        let mut seg_start = -1i32;
        for i in start..end {
            if !point_deleted[i as usize] {
                if seg_start < 0 {
                    seg_start = write;
                }
                write += 1;
            } else if seg_start >= 0 {
                out.push((seg_start, write));
                seg_start = -1;
            }
        }
        if seg_start >= 0 {
            out.push((seg_start, write));
        }
    }
    out
}
