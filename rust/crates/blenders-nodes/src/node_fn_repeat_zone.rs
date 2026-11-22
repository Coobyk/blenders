/// Port of `function/nodes/node_fn_repeat_zone.cc` — node `node_fn_repeat_zone`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
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
/// `exec` (verbatim — repeat body run
/// `iterations` times threading state).
pub fn exec(iterations: i64, mut state: Vec<f32>, body: &mut dyn FnMut(&mut Vec<f32>)) {
    for _ in 0..iterations.max(0) {
        body(&mut state);
    }
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
