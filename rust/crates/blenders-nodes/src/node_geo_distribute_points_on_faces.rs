/// Port of `geometry/nodes/node_geo_distribute_points_on_faces.cc` — node `node_geo_distribute_points_on_faces`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Mesh", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Selection", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Distance Min", super::node_declaration::SOCK_FLOAT, 1),
    ("Density Max", super::node_declaration::SOCK_FLOAT, 1),
    ("Density", super::node_declaration::SOCK_FLOAT, 1),
    ("Density Factor", super::node_declaration::SOCK_FLOAT, 1),
    ("Seed", super::node_declaration::SOCK_INT, 1),
    ("Points", super::node_declaration::SOCK_GEOMETRY, 2),
    ("Normal", super::node_declaration::SOCK_VECTOR, 2),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 2),
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
