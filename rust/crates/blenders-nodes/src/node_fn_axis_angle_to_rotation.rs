/// Port of `function/nodes/node_fn_axis_angle_to_rotation.cc` — node `node_fn_axis_angle_to_rotation`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Axis", super::node_declaration::SOCK_VECTOR, 1),
    ("Angle", super::node_declaration::SOCK_FLOAT, 1),
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
/// `exec` (verbatim — axis+angle → quat).
pub fn exec(axis: [f32; 3], angle: f32) -> [f32; 4] {
    let l = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    if l < 1e-6 {
        return [1.0, 0.0, 0.0, 0.0];
    }
    let h = angle * 0.5;
    let s = h.sin() / l;
    [h.cos(), axis[0] * s, axis[1] * s, axis[2] * s]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
