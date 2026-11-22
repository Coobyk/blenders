/// Port of `geometry/nodes/node_geo_evaluate_at_index.cc` — node `node_geo_evaluate_at_index`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Index", super::node_declaration::SOCK_INT, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — field gather:
/// values[clamp(index)]).
pub fn exec(values: &[f32], index: i64) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values[index.clamp(0, values.len() as i64 - 1) as usize]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
