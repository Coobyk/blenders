/// Port of `shader/nodes/node_shader_light_path.cc` — node `node_shader_light_path`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Is Camera Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Shadow Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Diffuse Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Glossy Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Singular Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Reflection Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Transmission Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Volume Scatter Ray", super::node_declaration::SOCK_FLOAT, 2),
    ("Ray Length", super::node_declaration::SOCK_FLOAT, 2),
    ("Ray Depth", super::node_declaration::SOCK_FLOAT, 2),
    ("Diffuse Depth", super::node_declaration::SOCK_FLOAT, 2),
    ("Glossy Depth", super::node_declaration::SOCK_FLOAT, 2),
    ("Transparent Depth", super::node_declaration::SOCK_FLOAT, 2),
    ("Transmission Depth", super::node_declaration::SOCK_FLOAT, 2),
    ("Portal Depth", super::node_declaration::SOCK_FLOAT, 2),
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
