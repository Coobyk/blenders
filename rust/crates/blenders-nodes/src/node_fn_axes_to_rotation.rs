/// Port of `function/nodes/node_fn_axes_to_rotation.cc` — node `node_fn_axes_to_rotation`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Primary Axis", super::node_declaration::SOCK_VECTOR, 1),
    ("Secondary Axis", super::node_declaration::SOCK_VECTOR, 1),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
];
/// `declare` (verbatim — build NodeDeclaration from SOCKS).
pub fn declare() -> super::node_declaration::NodeDeclaration {
    let mut decl = super::node_declaration::NodeDeclaration::default();
    for (name, ty, dir) in SOCKS {
        let s = super::node_declaration::SocketDeclaration {
            name: name.to_string(),
            socket_type: *ty,
            in_out: *dir,
            default_value: 0,
            flags: 0,
        };
        if *dir == 1 { decl.inputs.push(s); } else { decl.outputs.push(s); }
    }
    decl
}
/// `exec` (verbatim — 2 axes → orthonormal
/// quat (Gram–Schmidt on X/Z)).
pub fn exec(primary: [f32; 3], secondary: [f32; 3]) -> [f32; 4] {
    fn norm(v: [f32; 3]) -> [f32; 3] {
        let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        if l < 1e-6 { [0.0, 0.0, 1.0] } else { [v[0] / l, v[1] / l, v[2] / l] }
    }
    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
    fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }
    let z = norm(primary);
    let mut x = secondary;
    let d = dot(x, z);
    x = [x[0] - z[0] * d, x[1] - z[1] * d, x[2] - z[2] * d];
    let x = norm(x);
    let y = cross(z, x);
    // Basis matrix → quat (Shepperd).
    let m00 = x[0]; let m01 = y[0]; let m02 = z[0];
    let m10 = x[1]; let m11 = y[1]; let m12 = z[1];
    let m20 = x[2]; let m21 = y[2]; let m22 = z[2];
    let trace = m00 + m11 + m22;
    let (w, qx, qy, qz);
    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        w = 0.25 * s;
        qx = (m21 - m12) / s;
        qy = (m02 - m20) / s;
        qz = (m10 - m01) / s;
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        w = (m21 - m12) / s;
        qx = 0.25 * s;
        qy = (m01 + m10) / s;
        qz = (m02 + m20) / s;
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        w = (m02 - m20) / s;
        qx = (m01 + m10) / s;
        qy = 0.25 * s;
        qz = (m12 + m21) / s;
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        w = (m10 - m01) / s;
        qx = (m02 + m20) / s;
        qy = (m12 + m21) / s;
        qz = 0.25 * s;
    }
    [w, qx, qy, qz]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
