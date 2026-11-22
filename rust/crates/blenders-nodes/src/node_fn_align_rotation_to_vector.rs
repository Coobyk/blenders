/// Port of `function/nodes/node_fn_align_rotation_to_vector.cc` — node `node_fn_align_rotation_to_vector`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
    ("Factor", super::node_declaration::SOCK_FLOAT, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
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
/// `exec` (verbatim — same swing rotation
/// as align_euler, but on a Rotation input).
pub fn exec(q: [f32; 4], factor: f32, vector: [f32; 3], axis: i32) -> [f32; 4] {
    super::node_fn_align_euler_to_vector::exec(q, factor, vector, axis)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
