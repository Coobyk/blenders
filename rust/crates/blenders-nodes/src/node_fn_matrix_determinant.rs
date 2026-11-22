/// Port of `function/nodes/node_fn_matrix_determinant.cc` — node `node_fn_matrix_determinant`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Matrix", super::node_declaration::SOCK_MATRIX, 1),
    ("Determinant", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — 4x4 determinant via
/// cofactor expansion).
pub fn exec(m: &[[f32; 4]; 4]) -> f32 {
    fn det3(m: &[[f32; 4]; 4], r: &[usize; 3], c: &[usize; 3]) -> f32 {
        m[r[0]][c[0]] * (m[r[1]][c[1]] * m[r[2]][c[2]] - m[r[1]][c[2]] * m[r[2]][c[1]])
            - m[r[0]][c[1]] * (m[r[1]][c[0]] * m[r[2]][c[2]] - m[r[1]][c[2]] * m[r[2]][c[0]])
            + m[r[0]][c[2]] * (m[r[1]][c[0]] * m[r[2]][c[1]] - m[r[1]][c[1]] * m[r[2]][c[0]])
    }
    det3(m, &[1, 2, 3], &[1, 2, 3]) * m[0][0]
        - det3(m, &[1, 2, 3], &[0, 2, 3]) * m[0][1]
        + det3(m, &[1, 2, 3], &[0, 1, 3]) * m[0][2]
        - det3(m, &[1, 2, 3], &[0, 1, 2]) * m[0][3]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
