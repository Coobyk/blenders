/// Port of `function/nodes/node_fn_separate_transform.cc` — node `node_fn_separate_transform`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Matrix", super::node_declaration::SOCK_MATRIX, 1),
    ("Translation", super::node_declaration::SOCK_VECTOR, 2),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 2),
    ("Scale", super::node_declaration::SOCK_VECTOR, 2),
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
/// `exec` (verbatim — mat4 → TRS:
/// column lengths = scale, orthonormalized
/// basis → quat).
pub fn exec(m: &[[f32; 4]; 4]) -> ([f32; 3], [f32; 4], [f32; 3]) {
    let t = [m[3][0], m[3][1], m[3][2]];
    fn clen(m: &[[f32; 4]; 4], c: usize) -> f32 {
        (m[c][0] * m[c][0] + m[c][1] * m[c][1] + m[c][2] * m[c][2]).sqrt()
    }
    let s = [clen(m, 0), clen(m, 1), clen(m, 2)];
    let q = crate::node_fn_matrix_to_rotation::exec(m);
    (t, q, s)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
