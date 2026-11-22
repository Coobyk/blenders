/// Port of `function/nodes/node_fn_combine_xyz.cc` — node `node_fn_combine_xyz`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("X", super::node_declaration::SOCK_FLOAT, 1),
    ("Y", super::node_declaration::SOCK_FLOAT, 1),
    ("Z", super::node_declaration::SOCK_FLOAT, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 2),
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
/// `exec` (verbatim — pack 3 floats).
pub fn exec(x: f32, y: f32, z: f32) -> [f32; 3] {
    [x, y, z]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
