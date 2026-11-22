/// Port of `composite/nodes/node_composite_channel_key.cc` — node `node_composite_channel_key`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Image", super::node_declaration::SOCK_RGBA, 1),
    ("Image", super::node_declaration::SOCK_RGBA, 2),
    ("Matte", super::node_declaration::SOCK_FLOAT, 2),
    ("Minimum", super::node_declaration::SOCK_FLOAT, 1),
    ("Maximum", super::node_declaration::SOCK_FLOAT, 1),
    ("Color Space", super::node_declaration::SOCK_MENU, 1),
    ("RGB Key Channel", super::node_declaration::SOCK_MENU, 1),
    ("HSV Key Channel", super::node_declaration::SOCK_MENU, 1),
    ("YUV Key Channel", super::node_declaration::SOCK_MENU, 1),
    ("YCbCr Key Channel", super::node_declaration::SOCK_MENU, 1),
    ("Limit Method", super::node_declaration::SOCK_MENU, 1),
    ("RGB Limit Channel", super::node_declaration::SOCK_MENU, 1),
    ("HSV Limit Channel", super::node_declaration::SOCK_MENU, 1),
    ("YUV Limit Channel", super::node_declaration::SOCK_MENU, 1),
    ("YCbCr Limit Channel", super::node_declaration::SOCK_MENU, 1),
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
