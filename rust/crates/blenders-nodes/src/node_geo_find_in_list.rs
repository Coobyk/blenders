/// Port of `geometry/nodes/node_geo_find_in_list.cc` — node `node_geo_find_in_list`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Index", super::node_declaration::SOCK_INT, 2),
    ("Exists", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `exec` (verbatim — first index where
/// `list[i] == value`; -1 if absent).
pub fn exec(list: &[f32], value: f32) -> (i64, bool) {
    for (i, v) in list.iter().enumerate() {
        if *v == value {
            return (i as i64, true);
        }
    }
    (-1, false)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
