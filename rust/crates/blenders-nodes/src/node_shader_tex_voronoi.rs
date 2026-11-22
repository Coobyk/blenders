/// Port of `shader/nodes/node_shader_tex_voronoi.cc` — node `node_shader_tex_voronoi`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
    ("W", super::node_declaration::SOCK_FLOAT, 1),
    ("Scale", super::node_declaration::SOCK_FLOAT, 1),
    ("Detail", super::node_declaration::SOCK_FLOAT, 1),
    ("Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Lacunarity", super::node_declaration::SOCK_FLOAT, 1),
    ("Smoothness", super::node_declaration::SOCK_FLOAT, 1),
    ("Exponent", super::node_declaration::SOCK_FLOAT, 1),
    ("Randomness", super::node_declaration::SOCK_FLOAT, 1),
    ("Distance", super::node_declaration::SOCK_FLOAT, 2),
    ("Color", super::node_declaration::SOCK_RGBA, 2),
    ("Position", super::node_declaration::SOCK_VECTOR, 2),
    ("W", super::node_declaration::SOCK_FLOAT, 2),
    ("Radius", super::node_declaration::SOCK_FLOAT, 2),
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
/// `update` (verbatim — socket-level update hook; per-node body pending).
pub fn update() {}
/// `exec` (verbatim — per-node geometry/shader exec; PORT_PENDING on engine).
pub fn exec() {}
