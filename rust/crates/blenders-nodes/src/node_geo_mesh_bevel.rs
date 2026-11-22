/// Port of `geometry/nodes/node_geo_mesh_bevel.cc` — node `node_geo_mesh_bevel`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Mesh", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Mesh", super::node_declaration::SOCK_GEOMETRY, 2),
    ("Selection", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Affect Kind", super::node_declaration::SOCK_MENU, 1),
    ("Start Left Offset", super::node_declaration::SOCK_FLOAT, 1),
    ("Start Right Offset", super::node_declaration::SOCK_FLOAT, 1),
    ("End Left Offset", super::node_declaration::SOCK_FLOAT, 1),
    ("End Right Offset", super::node_declaration::SOCK_FLOAT, 1),
    ("Offset", super::node_declaration::SOCK_FLOAT, 1),
    ("Miter", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Spread", super::node_declaration::SOCK_FLOAT, 1),
    ("Segments", super::node_declaration::SOCK_INT, 1),
    ("Shape", super::node_declaration::SOCK_FLOAT, 1),
    ("Profile", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Vertex Face", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Edge Face", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Outer Edge", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Mid Edge", super::node_declaration::SOCK_BOOLEAN, 2),
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
