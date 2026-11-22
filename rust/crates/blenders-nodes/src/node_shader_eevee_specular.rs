/// Port of `shader/nodes/node_shader_eevee_specular.cc` — node `node_shader_eevee_specular`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Base Color", super::node_declaration::SOCK_RGBA, 1),
    ("Specular", super::node_declaration::SOCK_RGBA, 1),
    ("Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Emissive Color", super::node_declaration::SOCK_RGBA, 1),
    ("Transparency", super::node_declaration::SOCK_FLOAT, 1),
    ("Normal", super::node_declaration::SOCK_VECTOR, 1),
    ("Clear Coat", super::node_declaration::SOCK_FLOAT, 1),
    ("Clear Coat Roughness", super::node_declaration::SOCK_FLOAT, 1),
    ("Clear Coat Normal", super::node_declaration::SOCK_VECTOR, 1),
    ("Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("BSDF", super::node_declaration::SOCK_SHADER, 2),
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
