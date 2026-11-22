/// Port of `geometry/nodes/node_geo_mesh_topology_corners_of_face.cc` — node `node_geo_mesh_topology_corners_of_face`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Face Index", super::node_declaration::SOCK_INT, 1),
    ("Weights", super::node_declaration::SOCK_FLOAT, 1),
    ("Sort Index", super::node_declaration::SOCK_INT, 1),
    ("Corner Index", super::node_declaration::SOCK_INT, 2),
    ("Total", super::node_declaration::SOCK_INT, 2),
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
