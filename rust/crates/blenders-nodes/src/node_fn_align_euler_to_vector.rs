/// Port of `function/nodes/node_fn_align_euler_to_vector.cc` — node `node_fn_align_euler_to_vector`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Rotation", super::node_declaration::SOCK_ROTATION, 1),
    ("Factor", super::node_declaration::SOCK_FLOAT, 1),
    ("Vector", super::node_declaration::SOCK_VECTOR, 1),
    ("Rotation", super::node_declaration::SOCK_ROTATION, 2),
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
/// `exec` (verbatim — swing-rotate `eul`'s
/// `axis` onto `vector` by `factor`).
/// Ported from `alignEulerToVector` math.
pub fn exec(eul_quat: [f32; 4], factor: f32, vector: [f32; 3], axis: i32) -> [f32; 4] {
    let vlen = (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt();
    if vlen < 1e-6 {
        return eul_quat;
    }
    let target = [vector[0] / vlen, vector[1] / vlen, vector[2] / vlen];
    // Local axis rotated by current quat.
    let local = match axis {
        0 => [1.0, 0.0, 0.0],
        1 => [0.0, 1.0, 0.0],
        _ => [0.0, 0.0, 1.0],
    };
    let (w, x, y, z) = (eul_quat[0], eul_quat[1], eul_quat[2], eul_quat[3]);
    let t = [
        2.0 * (y * local[2] - z * local[1]),
        2.0 * (z * local[0] - x * local[2]),
        2.0 * (x * local[1] - y * local[0]),
    ];
    let cur = [
        local[0] + w * t[0] + y * t[2] - z * t[1],
        local[1] + w * t[1] + z * t[0] - x * t[2],
        local[2] + w * t[2] + x * t[1] - y * t[0],
    ];
    // rotation_between_vecs_to_quat.
    let c = cur[0] * target[0] + cur[1] * target[1] + cur[2] * target[2];
    let axis_v = [
        cur[1] * target[2] - cur[2] * target[1],
        cur[2] * target[0] - cur[0] * target[2],
        cur[0] * target[1] - cur[1] * target[0],
    ];
    let alen = (axis_v[0] * axis_v[0] + axis_v[1] * axis_v[1] + axis_v[2] * axis_v[2]).sqrt();
    let angle = c.clamp(-1.0, 1.0).acos() * factor.clamp(0.0, 1.0);
    if alen < 1e-6 {
        return eul_quat;
    }
    let hs = (angle * 0.5).sin() / alen;
    let delta = [
        (angle * 0.5).cos(),
        axis_v[0] * hs,
        axis_v[1] * hs,
        axis_v[2] * hs,
    ];
    // Premultiply: delta * q.
    [
        delta[0] * w - delta[1] * x - delta[2] * y - delta[3] * z,
        delta[0] * x + delta[1] * w + delta[2] * z - delta[3] * y,
        delta[0] * y - delta[1] * z + delta[2] * w + delta[3] * x,
        delta[0] * z + delta[1] * y - delta[2] * x + delta[3] * w,
    ]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
