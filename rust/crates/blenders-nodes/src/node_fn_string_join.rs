/// Port of `function/nodes/node_fn_string_join.cc` — node `node_fn_string_join`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Delimiter", super::node_declaration::SOCK_STRING, 1),
    ("Strings", super::node_declaration::SOCK_STRING, 1),
    ("String", super::node_declaration::SOCK_STRING, 2),
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
/// `exec` (verbatim — join field with
/// delimiter).
pub fn exec(delimiter: &str, strings: &[String]) -> String {
    strings.join(delimiter)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
