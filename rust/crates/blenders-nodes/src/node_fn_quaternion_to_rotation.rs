/// Port of `function/nodes/node_fn_quaternion_to_rotation.cc` — node `node_fn_quaternion_to_rotation`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("W", super::node_declaration::SOCK_FLOAT, 1),
    ("X", super::node_declaration::SOCK_FLOAT, 1),
    ("Y", super::node_declaration::SOCK_FLOAT, 1),
    ("Z", super::node_declaration::SOCK_FLOAT, 1),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 2),
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
/// `exec` (verbatim — pack + normalize quat).
pub fn exec(w: f32, x: f32, y: f32, z: f32) -> [f32; 4] {
    let l = (w * w + x * x + y * y + z * z).sqrt();
    if l < 1e-6 {
        [1.0, 0.0, 0.0, 0.0]
    } else {
        [w / l, x / l, y / l, z / l]
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
