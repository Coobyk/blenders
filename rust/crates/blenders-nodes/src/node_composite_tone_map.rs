/// Port of `composite/nodes/node_composite_tone_map.cc` — node `node_composite_tone_map`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Image", super::node_declaration::SOCK_RGBA, 1),
    ("Image", super::node_declaration::SOCK_RGBA, 2),
    ("Type", super::node_declaration::SOCK_MENU, 1),
    ("Key", super::node_declaration::SOCK_FLOAT, 1),
    ("Balance", super::node_declaration::SOCK_FLOAT, 1),
    ("Gamma", super::node_declaration::SOCK_FLOAT, 1),
    ("Intensity", super::node_declaration::SOCK_FLOAT, 1),
    ("Contrast", super::node_declaration::SOCK_FLOAT, 1),
    ("Light Adaptation", super::node_declaration::SOCK_FLOAT, 1),
    ("Chromatic Adaptation", super::node_declaration::SOCK_FLOAT, 1),
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
