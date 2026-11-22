/// Port of `intern/smooth_curves.cc` —
/// gaussian-smooth curve points.

/// `smooth_curves` (verbatim — 1-2-1 kernel
// pass × iterations).
pub fn smooth_curves(
    positions: &mut [[f32; 3]],
    points_by_curve: &[(i32, i32)],
    iterations: i32,
    factor: f32,
) {
    for _ in 0..iterations {
        let src: Vec<[f32; 3]> = positions.to_vec();
        for &(s, e) in points_by_curve {
            for i in (s + 1)..(e - 1) {
                let a = src[(i - 1) as usize];
                let b = src[i as usize];
                let c = src[(i + 1) as usize];
                for k in 0..3 {
                    let smooth = (a[k] + b[k] * 2.0 + c[k]) * 0.25;
                    positions[i as usize][k] = b[k] + (smooth - b[k]) * factor;
                }
            }
        }
    }
}
