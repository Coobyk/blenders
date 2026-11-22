/// Port of `function/nodes/node_fn_match_string.cc` — node `node_fn_match_string`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("String", super::node_declaration::SOCK_STRING, 1),
    ("Key", super::node_declaration::SOCK_STRING, 1),
    ("Result", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `KeyMapOperation` (verbatim —
/// starts_with / ends_with / contains /
/// equals).
pub const KEYMAP_OPERATION_STARTS_WITH: i32 = 0;
pub const KEYMAP_OPERATION_ENDS_WITH: i32 = 1;
pub const KEYMAP_OPERATION_CONTAINS: i32 = 2;
pub const KEYMAP_OPERATION_EQUALS: i32 = 3;
/// `exec` (verbatim — match per op).
pub fn exec(op: i32, s: &str, key: &str) -> bool {
    match op {
        KEYMAP_OPERATION_ENDS_WITH => s.ends_with(key),
        KEYMAP_OPERATION_CONTAINS => s.contains(key),
        KEYMAP_OPERATION_EQUALS => s == key,
        // KEYMAP_OPERATION_STARTS_WITH.
        _ => s.starts_with(key),
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
