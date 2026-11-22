/// Port of `function/nodes/node_fn_combine_transform.cc` — node `node_fn_combine_transform`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Translation", super::node_declaration::SOCK_VECTOR, 1),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
    ("Scale", super::node_declaration::SOCK_VECTOR, 1),
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
/// `exec` (verbatim — TRS → mat4).
pub fn exec(t: [f32; 3], q: [f32; 4], s: [f32; 3]) -> [[f32; 4]; 4] {
    let (w, x, y, z) = (q[0], q[1], q[2], q[3]);
    // Quat → 3x3.
    let r = [
        [1.0 - 2.0 * (y * y + z * z), 2.0 * (x * y - w * z), 2.0 * (x * z + w * y)],
        [2.0 * (x * y + w * z), 1.0 - 2.0 * (x * x + z * z), 2.0 * (y * z - w * x)],
        [2.0 * (x * z - w * y), 2.0 * (y * z + w * x), 1.0 - 2.0 * (x * x + y * y)],
    ];
    [
        [r[0][0] * s[0], r[1][0] * s[0], r[2][0] * s[0], 0.0],
        [r[0][1] * s[1], r[1][1] * s[1], r[2][1] * s[1], 0.0],
        [r[0][2] * s[2], r[1][2] * s[2], r[2][2] * s[2], 0.0],
        [t[0], t[1], t[2], 1.0],
    ]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
