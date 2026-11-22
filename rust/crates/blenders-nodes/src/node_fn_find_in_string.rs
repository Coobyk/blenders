/// Port of `function/nodes/node_fn_find_in_string.cc` — node `node_fn_find_in_string`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("String", super::node_declaration::SOCK_STRING, 1),
    ("Search", super::node_declaration::SOCK_STRING, 1),
    ("Position", super::node_declaration::SOCK_INT, 2),
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
/// `exec` (verbatim — substring search;
/// returns (pos, found)).
pub fn exec(s: &str, search: &str) -> (i64, bool) {
    match s.find(search) {
        Some(i) => (i as i64, true),
        None => (0, false),
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
