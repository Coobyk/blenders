//! Port of `intern/simplify_curves.cc` —
//! Ramer-Douglas-Peucker polyline simplification.

/// `perpendicular_distance` (verbatim — point
/// to segment).
pub fn perpendicular_distance(p: [f32; 3], a: [f32; 3], b: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    if len2 == 0.0 {
        return (ap[0] * ap[0] + ap[1] * ap[1] + ap[2] * ap[2]).sqrt();
    }
    let t = ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len2).clamp(0.0, 1.0);
    let cx = a[0] + ab[0] * t;
    let cy = a[1] + ab[1] * t;
    let cz = a[2] + ab[2] * t;
    ((p[0] - cx).powi(2) + (p[1] - cy).powi(2) + (p[2] - cz).powi(2)).sqrt()
}
/// `ramer_douglas_peucker` (verbatim — keep
/// endpoints, recurse on farthest).
pub fn ramer_douglas_peucker(
    positions: &[[f32; 3]],
    start: usize,
    end: usize,
    epsilon: f32,
    keep: &mut [bool],
) {
    if end <= start + 1 {
        return;
    }
    let a = positions[start];
    let b = positions[end];
    let mut max_d = -1.0f32;
    let mut max_i = start;
    for i in (start + 1)..end {
        let d = perpendicular_distance(positions[i], a, b);
        if d > max_d {
            max_d = d;
            max_i = i;
        }
    }
    if max_d > epsilon {
        keep[max_i] = true;
        ramer_douglas_peucker(positions, start, max_i, epsilon, keep);
        ramer_douglas_peucker(positions, max_i, end, epsilon, keep);
    }
}
/// `curve_simplify` (verbatim — full point mask).
pub fn curve_simplify(positions: &[[f32; 3]], epsilon: f32) -> Vec<bool> {
    let n = positions.len();
    let mut keep = vec![false; n];
    if n > 0 {
        keep[0] = true;
        keep[n - 1] = true;
        ramer_douglas_peucker(positions, 0, n - 1, epsilon, &mut keep);
    }
    keep
}
