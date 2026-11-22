/// Port of `geometry/nodes/node_geo_xpbd_solver.cc` — node `node_geo_xpbd_solver`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("World", super::node_declaration::SOCK_BUNDLE, 1),
    ("World", super::node_declaration::SOCK_BUNDLE, 2),
    ("Delta Time", super::node_declaration::SOCK_FLOAT, 1),
    ("Filter", super::node_declaration::SOCK_STRING, 1),
    ("Simulation to World", super::node_declaration::SOCK_MATRIX, 1),
    ("Substeps", super::node_declaration::SOCK_INT, 1),
    ("Constraint Iterations", super::node_declaration::SOCK_INT, 1),
    ("Solver Path", super::node_declaration::SOCK_STRING, 1),
    ("Begin", super::node_declaration::SOCK_FLOAT, 1),
    ("End", super::node_declaration::SOCK_FLOAT, 1),
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
