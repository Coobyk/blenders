/// Port of `function/nodes/node_fn_combine_matrix.cc` — node `node_fn_combine_matrix`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Input Values", super::node_declaration::SOCK_FLOAT, 1),
    ("Matrix", super::node_declaration::SOCK_MATRIX, 2),
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
/// `exec` (verbatim — 16 floats → mat4).
pub fn exec(v: &[f32; 16]) -> [[f32; 4]; 4] {
    let mut m = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            m[i][j] = v[i * 4 + j];
        }
    }
    m
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
