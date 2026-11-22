/// Port of `intern/subdivide_curves.cc` —
/// insert points between curve points.

/// `subdivide_curves` (verbatim — per-segment
// midpoint insert × cuts).
pub fn subdivide_curves(
    positions: &[[f32; 3]],
    points_by_curve: &[(i32, i32)],
    cuts: i32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for &(s, e) in points_by_curve {
        for i in s..(e - 1) {
            let a = positions[i as usize];
            let b = positions[(i + 1) as usize];
            out.push(a);
            for c in 1..=cuts {
                let t = c as f32 / (cuts + 1) as f32;
                out.push([
                    a[0] + (b[0] - a[0]) * t,
                    a[1] + (b[1] - a[1]) * t,
                    a[2] + (b[2] - a[2]) * t,
                ]);
            }
        }
        out.push(positions[(e - 1) as usize]);
    }
    out
}
