/// Port of `shader/nodes/node_shader_volume_coefficients.cc` — node `node_shader_volume_coefficients`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Volume", super::node_declaration::SOCK_SHADER, 2),
    ("Weight", super::node_declaration::SOCK_FLOAT, 1),
    ("Absorption Coefficients", super::node_declaration::SOCK_VECTOR, 1),
    ("Scatter Coefficients", super::node_declaration::SOCK_VECTOR, 1),
    ("Anisotropy", super::node_declaration::SOCK_FLOAT, 1),
    ("IOR", super::node_declaration::SOCK_FLOAT, 1),
    ("Backscatter", super::node_declaration::SOCK_FLOAT, 1),
    ("Alpha", super::node_declaration::SOCK_FLOAT, 1),
    ("Diameter", super::node_declaration::SOCK_FLOAT, 1),
    ("Emission Coefficients", super::node_declaration::SOCK_VECTOR, 1),
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
