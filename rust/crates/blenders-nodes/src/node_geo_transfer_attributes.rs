/// Port of `geometry/nodes/node_geo_transfer_attributes.cc` — node `node_geo_transfer_attributes`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Target", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Target", super::node_declaration::SOCK_GEOMETRY, 2),
    ("Transferred Names", super::node_declaration::SOCK_STRING, 2),
    ("Target Point ID", super::node_declaration::SOCK_INT, 1),
    ("Target Edge ID", super::node_declaration::SOCK_INT, 1),
    ("Target Face ID", super::node_declaration::SOCK_INT, 1),
    ("Target Corner ID", super::node_declaration::SOCK_INT, 1),
    ("Target Curve ID", super::node_declaration::SOCK_INT, 1),
    ("Target Instance ID", super::node_declaration::SOCK_INT, 1),
    ("Source", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Source Point ID", super::node_declaration::SOCK_INT, 1),
    ("Source Edge ID", super::node_declaration::SOCK_INT, 1),
    ("Source Face ID", super::node_declaration::SOCK_INT, 1),
    ("Source Corner ID", super::node_declaration::SOCK_INT, 1),
    ("Source Curve ID", super::node_declaration::SOCK_INT, 1),
    ("Source Instance ID", super::node_declaration::SOCK_INT, 1),
    ("Pattern Mode", super::node_declaration::SOCK_MENU, 1),
    ("Attribute Names", super::node_declaration::SOCK_STRING, 1),
    ("Exclude Names", super::node_declaration::SOCK_BOOLEAN, 1),
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
