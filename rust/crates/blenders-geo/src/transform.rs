/// Port of `intern/transform.cc` — Geometry
/// Transform node (translate/rotate/scale on
/// every component).

/// `transform_geometry` (verbatim — 4×4 on
/// positions).
pub fn transform_positions(
    positions: &mut [[f32; 3]],
    m: [[f32; 4]; 4],
) {
    for p in positions.iter_mut() {
        let x = p[0];
        let y = p[1];
        let z = p[2];
        p[0] = m[0][0] * x + m[0][1] * y + m[0][2] * z + m[0][3];
        p[1] = m[1][0] * x + m[1][1] * y + m[1][2] * z + m[1][3];
        p[2] = m[2][0] * x + m[2][1] * y + m[2][2] * z + m[2][3];
    }
}
/// `transform_normals` (verbatim — inverse-
/// transpose for direction vectors).
pub fn transform_normals(
    normals: &mut [[f32; 3]],
    m: [[f32; 4]; 4],
) {
    for n in normals.iter_mut() {
        let x = n[0];
        let y = n[1];
        let z = n[2];
        n[0] = m[0][0] * x + m[0][1] * y + m[0][2] * z;
        n[1] = m[1][0] * x + m[1][1] * y + m[1][2] * z;
        n[2] = m[2][0] * x + m[2][1] * y + m[2][2] * z;
        let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        if l > 0.0 {
            n[0] /= l;
            n[1] /= l;
            n[2] /= l;
        }
    }
}
