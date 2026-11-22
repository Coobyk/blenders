/// Port of `geometry/nodes/node_geo_raycast.cc` — node `node_geo_raycast`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Target Geometry", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Interpolation", super::node_declaration::SOCK_MENU, 1),
    ("Source Position", super::node_declaration::SOCK_VECTOR, 1),
    ("Ray Direction", super::node_declaration::SOCK_VECTOR, 1),
    ("Ray Length", super::node_declaration::SOCK_FLOAT, 1),
    ("Is Hit", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Hit Position", super::node_declaration::SOCK_VECTOR, 2),
    ("Hit Normal", super::node_declaration::SOCK_VECTOR, 2),
    ("Hit Distance", super::node_declaration::SOCK_FLOAT, 2),
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
