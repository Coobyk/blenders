/// Port of `intern/extend_curves.cc` — extend
/// curve endpoints by tangent distance.

/// `extend_curves` (verbatim — project along
// unit tangent by distance).
pub fn extend_curves(
    positions: &mut Vec<[f32; 3]>,
    points_by_curve: &[(i32, i32)],
    start_len: f32,
    end_len: f32,
) {
    for &(s, e) in points_by_curve {
        let n = (e - s) as usize;
        if n < 2 {
            continue;
        }
        // Start extension.
        let a = positions[s as usize];
        let b = positions[(s + 1) as usize];
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if l > 0.0 && start_len != 0.0 {
            let t = start_len / l;
            positions.insert(
                s as usize,
                [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t],
            );
        }
        let a = positions[(e - 1) as usize];
        let b = positions[(e - 2) as usize];
        let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        if l > 0.0 && end_len != 0.0 {
            let t = end_len / l;
            positions.insert(
                e as usize,
                [a[0] + d[0] * t, a[1] + d[1] * t, a[2] + d[2] * t],
            );
        }
    }
}
