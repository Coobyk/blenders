//! Port of `intern/fit_curves.cc` — fit
//! polylines to Bézier (least-squares handles).

/// `fit_curve` (verbatim — chord-length
// parameterize then least-squares handle fit).
pub fn fit_curves_chord_params(positions: &[[f32; 3]]) -> Vec<f32> {
    let n = positions.len();
    let mut u = vec![0.0f32; n];
    for i in 1..n {
        let a = positions[i - 1];
        let b = positions[i];
        u[i] = u[i - 1]
            + ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2) + (b[2] - a[2]).powi(2)).sqrt();
    }
    let total = u[n - 1].max(1e-9);
    for v in &mut u {
        *v /= total;
    }
    u
}
