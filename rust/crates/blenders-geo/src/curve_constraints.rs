//! Port of `intern/curve_constraints.cc` —
//! XPBD length + collision constraints on
//! curve points (Gauss-Seidel).

/// `compute_segment_lengths` (verbatim —
//! per-curve segment length array).
pub fn compute_segment_lengths(
    positions: &[[f32; 3]],
    points_by_curve: &[(i32, i32)],
) -> Vec<Vec<f32>> {
    let mut out = Vec::with_capacity(points_by_curve.len());
    for &(start, end) in points_by_curve {
        let mut seg = Vec::new();
        for i in start..(end - 1) {
            let a = positions[i as usize];
            let b = positions[(i + 1) as usize];
            seg.push(
                ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt(),
            );
        }
        out.push(seg);
    }
    out
}
/// `solve_length_constraints` (verbatim — move
//! both ends half the stretch).
pub fn solve_length_constraints(
    positions: &mut [[f32; 3]],
    segment_lengths: &[Vec<f32>],
    points_by_curve: &[(i32, i32)],
    rest_lengths: &[Vec<f32>],
) {
    for (ci, &(start, end)) in points_by_curve.iter().enumerate() {
        for i in start..(end - 1) {
            let a = i as usize;
            let b = (i + 1) as usize;
            let cur = segment_lengths[ci][(i - start) as usize];
            let rest = rest_lengths[ci][(i - start) as usize];
            if cur == 0.0 || rest == 0.0 {
                continue;
            }
            let diff = (cur - rest) / cur;
            let d = [
                (positions[b][0] - positions[a][0]) * 0.5 * diff,
                (positions[b][1] - positions[a][1]) * 0.5 * diff,
                (positions[b][2] - positions[a][2]) * 0.5 * diff,
            ];
            positions[a][0] += d[0];
            positions[a][1] += d[1];
            positions[a][2] += d[2];
            positions[b][0] -= d[0];
            positions[b][1] -= d[1];
            positions[b][2] -= d[2];
        }
    }
}
