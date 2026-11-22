/// Port of `geometry/nodes/node_geo_string_to_curves.cc` — node `node_geo_string_to_curves`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Curve Instances", super::node_declaration::SOCK_GEOMETRY, 2),
    ("Remainder", super::node_declaration::SOCK_STRING, 2),
    ("Line", super::node_declaration::SOCK_INT, 2),
    ("Word", super::node_declaration::SOCK_INT, 2),
    ("Pivot Point", super::node_declaration::SOCK_VECTOR, 2),
    ("String", super::node_declaration::SOCK_STRING, 1),
    ("Size", super::node_declaration::SOCK_FLOAT, 1),
    ("Font", super::node_declaration::SOCK_FLOAT, 1),
    ("Align X", super::node_declaration::SOCK_MENU, 1),
    ("Align Y", super::node_declaration::SOCK_MENU, 1),
    ("Pivot Point", super::node_declaration::SOCK_MENU, 1),
    ("Character Spacing", super::node_declaration::SOCK_FLOAT, 1),
    ("Word Spacing", super::node_declaration::SOCK_FLOAT, 1),
    ("Line Spacing", super::node_declaration::SOCK_FLOAT, 1),
    ("Overflow", super::node_declaration::SOCK_MENU, 1),
    ("Text Box Width", super::node_declaration::SOCK_FLOAT, 1),
    ("Text Box Height", super::node_declaration::SOCK_FLOAT, 1),
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
