/// Port of `function/nodes/node_fn_switch.cc` — node `node_fn_switch`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Switch", super::node_declaration::SOCK_BOOLEAN, 1),
    ("False", super::node_declaration::SOCK_FLOAT, 1),
    ("True", super::node_declaration::SOCK_FLOAT, 1),
    ("Output", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — select input by
/// switch bool).
pub fn exec(switch: bool, v_false: f32, v_true: f32) -> f32 {
    if switch { v_true } else { v_false }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
