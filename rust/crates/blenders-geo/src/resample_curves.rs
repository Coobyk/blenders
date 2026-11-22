/// Port of `intern/resample_curves.cc` —
/// uniform resample by count or length.

/// `resample_curves` (verbatim — walk
// cumulative arc-length, emit evenly-spaced
// samples).
pub fn resample_curves(
    positions: &[[f32; 3]],
    points_by_curve: &[(i32, i32)],
    counts: &[i32],
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for (ci, &(s, e)) in points_by_curve.iter().enumerate() {
        let n = (e - s) as usize;
        let count = counts[ci].max(2) as usize;
        // Arc-length table.
        let mut len = vec![0.0f32; n];
        for i in 1..n {
            let a = positions[(s as usize) + i - 1];
            let b = positions[(s as usize) + i];
            len[i] = len[i - 1]
                + ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2))
                    .sqrt();
        }
        let total = len[n - 1];
        for k in 0..count {
            let target = total * k as f32 / (count - 1) as f32;
            let mut seg = 0usize;
            while seg + 1 < n && len[seg + 1] < target {
                seg += 1;
            }
            let span = (len[seg + 1] - len[seg]).max(1e-9);
            let t = (target - len[seg]) / span;
            let a = positions[(s as usize) + seg];
            let b = positions[(s as usize) + seg + 1];
            out.push([
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]);
        }
    }
    out
}
