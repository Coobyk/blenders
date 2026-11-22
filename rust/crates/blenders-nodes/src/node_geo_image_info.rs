/// Port of `geometry/nodes/node_geo_image_info.cc` — node `node_geo_image_info`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Image", super::node_declaration::SOCK_IMAGE, 1),
    ("Frame", super::node_declaration::SOCK_INT, 1),
    ("Width", super::node_declaration::SOCK_INT, 2),
    ("Height", super::node_declaration::SOCK_INT, 2),
    ("Has Alpha", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Frame Count", super::node_declaration::SOCK_INT, 2),
    ("FPS", super::node_declaration::SOCK_FLOAT, 2),
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
