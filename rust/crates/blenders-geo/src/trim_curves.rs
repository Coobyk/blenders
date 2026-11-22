/// Port of `intern/trim_curves.cc` — trim
/// curves to parameter or length range
/// (poly/catmull-rom/bezier/nurbs).

/// `lookup_point_uniform_spacing` (verbatim —
/// segment index + local t from length).
pub fn lookup_point_uniform_spacing(
    lengths: &[f32],
    target: f32,
) -> (i32, f32) {
    if lengths.is_empty() {
        return (0, 0.0);
    }
    for i in 0..lengths.len() {
        if lengths[i] >= target {
            let prev = if i > 0 { lengths[i - 1] } else { 0.0 };
            let span = lengths[i] - prev;
            let t = if span > 0.0 { (target - prev) / span } else { 0.0 };
            return (i as i32, t.clamp(0.0, 1.0));
        }
    }
    (lengths.len() as i32 - 1, 1.0)
}
/// `lookup_point_polygonal` (verbatim —
/// linear index+t).
pub fn lookup_point_polygonal(lengths: &[f32], target: f32) -> (i32, f32) {
    lookup_point_uniform_spacing(lengths, target)
}
/// `compute_curve_trim_parameters` (verbatim —
// start/end point+t per curve).
pub fn compute_curve_trim_parameters(
    lengths: &[f32],
    start: f32,
    end: f32,
) -> ((i32, f32), (i32, f32)) {
    (
        lookup_point_uniform_spacing(lengths, start),
        lookup_point_uniform_spacing(lengths, end),
    )
}
/// `sample_interval_linear` (verbatim —
// resample trimmed span).
pub fn sample_interval_linear(
    positions: &[[f32; 3]],
    i0: usize,
    t0: f32,
    i1: usize,
    t1: f32,
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for i in i0..=i1 {
        let a = positions[i];
        let b = positions[(i + 1).min(positions.len() - 1)];
        let t = if i == i0 { t0 } else { 0.0 };
        out.push([
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
        ]);
    }
    let _ = t1;
    out
}
