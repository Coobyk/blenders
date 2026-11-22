/// Port of `shader/nodes/node_shader_volume_principled.cc` — node `node_shader_volume_principled`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Color", super::node_declaration::SOCK_RGBA, 1),
    ("Color Attribute", super::node_declaration::SOCK_STRING, 1),
    ("Density", super::node_declaration::SOCK_FLOAT, 1),
    ("Density Attribute", super::node_declaration::SOCK_STRING, 1),
    ("Anisotropy", super::node_declaration::SOCK_FLOAT, 1),
    ("Absorption Color", super::node_declaration::SOCK_RGBA, 1),
    ("Emission Strength", super::node_declaration::SOCK_FLOAT, 1),
    ("Emission Color", super::node_declaration::SOCK_RGBA, 1),
    ("Blackbody Intensity", super::node_declaration::SOCK_FLOAT, 1),
    ("Blackbody Tint", super::node_declaration::SOCK_RGBA, 1),
    ("Temperature", super::node_declaration::SOCK_FLOAT, 1),
    ("Temperature Attribute", super::node_declaration::SOCK_STRING, 1),
    ("Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Volume", super::node_declaration::SOCK_SHADER, 2),
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
