/// Port of `function/nodes/node_fn_slice_string.cc` — node `node_fn_slice_string`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("String", super::node_declaration::SOCK_STRING, 1),
    ("Position", super::node_declaration::SOCK_INT, 1),
    ("Length", super::node_declaration::SOCK_INT, 1),
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
/// `exec` (verbatim — slice s[pos..pos+len],
/// clamped like C's `StringRef::substr`).
pub fn exec(s: &str, pos: i64, len: i64) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len() as i64;
    let start = pos.clamp(0, n);
    let end = (pos + len.max(0)).clamp(start, n);
    chars[start as usize..end as usize].iter().collect()
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
