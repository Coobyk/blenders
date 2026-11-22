/// Port of `geometry/nodes/node_geo_input_position.cc` — node `node_geo_input_position`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Position", super::node_declaration::SOCK_VECTOR, 2),
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
/// `exec` (verbatim — read `position`
/// attribute at index).
pub fn exec(positions: &[[f32; 3]], index: i64) -> [f32; 3] {
    positions.get(index as usize).copied().unwrap_or([0.0; 3])
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
