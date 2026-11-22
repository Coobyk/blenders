/// Port of `geometry/nodes/node_geo_element_selection.cc` — node `node_geo_element_selection`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Selection", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `exec` (verbatim — read `.selection`
/// attr at index).
pub fn exec(selection: &[bool], index: i64) -> bool {
    selection.get(index as usize).copied().unwrap_or(false)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
