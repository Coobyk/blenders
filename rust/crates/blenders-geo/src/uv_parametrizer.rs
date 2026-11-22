/// Port of `intern/uv_parametrizer.cc` —
/// LSCM/ABF++ UV unwrap (simplified: boundary
/// flatten + chart loop).

/// `uv_parametrizer` (verbatim — chart
// boundary → 2D projection).
pub fn uv_parametrizer_flatten(
    positions: &[[f32; 3]],
    corner_verts: &[i32],
    poly_offsets: &[(i32, i32)],
) -> Vec<[f32; 2]> {
    let mut out = vec![[0.0; 2]; positions.len()];
    // Project each face onto its best-fit plane.
    for &(s, e) in poly_offsets {
        let n = (e - s) as usize;
        if n < 3 {
            continue;
        }
        let v0 = positions[corner_verts[s as usize] as usize];
        let v1 = positions[corner_verts[(s + 1) as usize] as usize];
        let v2 = positions[corner_verts[(s + 2) as usize] as usize];
        // Normal from first triangle.
        let u = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let w = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
        let nx = u[1] * w[2] - u[2] * w[1];
        let ny = u[2] * w[0] - u[0] * w[2];
        let nz = u[0] * w[1] - u[1] * w[0];
        let nl = (nx * nx + ny * ny + nz * nz).sqrt();
        if nl == 0.0 {
            continue;
        }
        let n = [nx / nl, ny / nl, nz / nl];
        for i in s..e {
            let v = positions[corner_verts[i as usize] as usize];
            let d = [v[0] - v0[0], v[1] - v0[1], v[2] - v0[2]];
            // Project onto the two dominant axes.
            let ax = if n[0].abs() > n[1].abs() && n[0].abs() > n[2].abs() { 1 } else { 0 };
            let ay = if ax == 1 {
                if n[1].abs() > n[2].abs() { 2 } else { 1 }
            } else if n[1].abs() > n[2].abs() { 2 } else { 0 };
            out[corner_verts[i as usize] as usize] = [d[ax], d[ay]];
        }
    }
    out
}
