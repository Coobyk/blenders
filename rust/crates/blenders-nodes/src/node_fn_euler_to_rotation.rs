/// Port of `function/nodes/node_fn_euler_to_rotation.cc` — node `node_fn_euler_to_rotation`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Euler", super::node_declaration::SOCK_VECTOR, 1),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
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
/// `exec` (verbatim — XYZ euler → quat).
pub fn exec(eul: [f32; 3]) -> [f32; 4] {
    // ZYX composite: qz * qy * qx (Blender XYZ order).
    let (x, y, z) = (eul[0] * 0.5, eul[1] * 0.5, eul[2] * 0.5);
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();
    [
        cx * cy * cz + sx * sy * sz,
        sx * cy * cz - cx * sy * sz,
        cx * sy * cz + sx * cy * sz,
        cx * cy * sz - sx * sy * cz,
    ]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
