/// Port of `composite/nodes/node_composite_mask.cc` — node `node_composite_mask`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Mask", super::node_declaration::SOCK_FLOAT, 2),
    ("Size Source", super::node_declaration::SOCK_MENU, 1),
    ("Size X", super::node_declaration::SOCK_INT, 1),
    ("Size Y", super::node_declaration::SOCK_INT, 1),
    ("Feather", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Motion Blur", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Samples", super::node_declaration::SOCK_INT, 1),
    ("Shutter", super::node_declaration::SOCK_FLOAT, 1),
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
