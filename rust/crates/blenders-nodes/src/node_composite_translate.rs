/// Port of `composite/nodes/node_composite_translate.cc` — node `node_composite_translate`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Image", super::node_declaration::SOCK_RGBA, 1),
    ("Image", super::node_declaration::SOCK_RGBA, 2),
    ("X", super::node_declaration::SOCK_FLOAT, 1),
    ("Y", super::node_declaration::SOCK_FLOAT, 1),
    ("Interpolation", super::node_declaration::SOCK_MENU, 1),
    ("Extension X", super::node_declaration::SOCK_MENU, 1),
    ("Extension Y", super::node_declaration::SOCK_MENU, 1),
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
