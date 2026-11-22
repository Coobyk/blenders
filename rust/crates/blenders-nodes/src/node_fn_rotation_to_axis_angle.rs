/// Port of `function/nodes/node_fn_rotation_to_axis_angle.cc` — node `node_fn_rotation_to_axis_angle`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
    ("Axis", super::node_declaration::SOCK_VECTOR, 2),
    ("Angle", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — quat → axis+angle).
pub fn exec(q: [f32; 4]) -> ([f32; 3], f32) {
    let s = (q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if s < 1e-6 {
        return ([0.0, 0.0, 1.0], 0.0);
    }
    (
        [q[1] / s, q[2] / s, q[3] / s],
        2.0 * (s.atan2(q[0])),
    )
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
