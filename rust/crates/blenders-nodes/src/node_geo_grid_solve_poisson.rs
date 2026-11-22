/// Port of `geometry/nodes/node_geo_grid_solve_poisson.cc` — node `node_geo_grid_solve_poisson`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Solution", super::node_declaration::SOCK_FLOAT, 2),
    ("Success", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Iterations", super::node_declaration::SOCK_INT, 2),
    ("Absolute Error", super::node_declaration::SOCK_FLOAT, 2),
    ("Relative Error", super::node_declaration::SOCK_FLOAT, 2),
    ("Grid", super::node_declaration::SOCK_FLOAT, 1),
    ("Max Iterations", super::node_declaration::SOCK_INT, 1),
    ("Error Threshold", super::node_declaration::SOCK_FLOAT, 1),
    ("Threshold Mode", super::node_declaration::SOCK_MENU, 1),
    ("Boundary", super::node_declaration::SOCK_MENU, 1),
    ("Boundary Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Boundary Gradient", super::node_declaration::SOCK_VECTOR, 1),
    ("Boundary Factor", super::node_declaration::SOCK_FLOAT, 1),
    ("Debug Boundary Value", super::node_declaration::SOCK_FLOAT, 2),
    ("Debug Boundary Gradient", super::node_declaration::SOCK_VECTOR, 2),
    ("Debug Boundary Factor", super::node_declaration::SOCK_FLOAT, 2),
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
