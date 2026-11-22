/// Port of `geometry/nodes/node_geo_curve_primitive_bezier_segment.cc` — node `node_geo_curve_primitive_bezier_segment`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Resolution", super::node_declaration::SOCK_INT, 1),
    ("Start", super::node_declaration::SOCK_VECTOR, 1),
    ("Start Handle", super::node_declaration::SOCK_VECTOR, 1),
    ("End Handle", super::node_declaration::SOCK_VECTOR, 1),
    ("End", super::node_declaration::SOCK_VECTOR, 1),
    ("Curve", super::node_declaration::SOCK_GEOMETRY, 2),
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
