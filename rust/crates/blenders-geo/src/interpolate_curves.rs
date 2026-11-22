/// Port of `intern/interpolate_curves.cc` —
/// evaluate one curve at another's parameter
/// positions.

/// `assign_samples_to_segments` (verbatim —
// target index per sample point).
pub fn assign_samples_to_segments(
    num_dst_points: i32,
    num_src_points: i32,
) -> Vec<i32> {
    let mut out = Vec::with_capacity(num_dst_points as usize);
    for i in 0..num_dst_points {
        out.push(i * (num_src_points - 1) / (num_dst_points - 1).max(1));
    }
    out
}
/// `sample_curve_padded` (verbatim — linear or
// catmull-rom interpolation per segment).
pub fn sample_curve_padded(
    positions: &[[f32; 3]],
    counts: &[i32],
) -> Vec<[f32; 3]> {
    let mut out = Vec::new();
    for &n in counts {
        for i in 0..n {
            let t = i as f32 / (n - 1).max(1) as f32;
            let src = (t * (positions.len() - 1) as f32) as usize;
            let next = (src + 1).min(positions.len() - 1);
            let ft = t * (positions.len() - 1) as f32 - src as f32;
            let a = positions[src];
            let b = positions[next];
            out.push([
                a[0] + (b[0] - a[0]) * ft,
                a[1] + (b[1] - a[1]) * ft,
                a[2] + (b[2] - a[2]) * ft,
            ]);
        }
    }
    out
}
/// `calculate_catmull_rom_basis_derivative`
/// (verbatim — derivative basis weights).
pub fn catmull_rom_basis_derivative(t: f32) -> [f32; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        -0.5 * (3.0 * t2 - 4.0 * t + 1.0),
        0.5 * (9.0 * t2 - 10.0 * t),
        -0.5 * (9.0 * t2 - 8.0 * t + 1.0),
        0.5 * (3.0 * t2 - 2.0 * t),
    ]
}
/// `get_catmull_rom_indices` (verbatim —
// 4-pt window clamped).
pub fn get_catmull_rom_indices(src_index: i32, num_src: i32) -> [i32; 4] {
    let i0 = (src_index - 1).max(0);
    let i1 = src_index.min(num_src - 1);
    let i2 = (src_index + 1).min(num_src - 1);
    let i3 = (src_index + 2).min(num_src - 1);
    [i0, i1, i2, i3]
}
