/// Port of `geometry/nodes/node_geo_curve_sample.cc` — node `node_geo_curve_sample`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Curves", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Factor", super::node_declaration::SOCK_FLOAT, 1),
    ("Length", super::node_declaration::SOCK_FLOAT, 1),
    ("Curve Index", super::node_declaration::SOCK_INT, 1),
    ("Position", super::node_declaration::SOCK_VECTOR, 2),
    ("Tangent", super::node_declaration::SOCK_VECTOR, 2),
    ("Normal", super::node_declaration::SOCK_VECTOR, 2),
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
