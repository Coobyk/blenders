/// Port of `composite/nodes/node_composite_glare.cc` — node `node_composite_glare`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Image", super::node_declaration::SOCK_RGBA, 1),
    ("Image", super::node_declaration::SOCK_RGBA, 2),
    ("Glare", super::node_declaration::SOCK_RGBA, 2),
    ("Highlights", super::node_declaration::SOCK_RGBA, 2),
    ("Type", super::node_declaration::SOCK_MENU, 1),
    ("Quality", super::node_declaration::SOCK_MENU, 1),
    ("Threshold", super::node_declaration::SOCK_FLOAT, 1),
    ("Smoothness", super::node_declaration::SOCK_FLOAT, 1),
    ("Clamp", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Maximum", super::node_declaration::SOCK_FLOAT, 1),
    ("Strength", super::node_declaration::SOCK_FLOAT, 1),
    ("Saturation", super::node_declaration::SOCK_FLOAT, 1),
    ("Tint", super::node_declaration::SOCK_RGBA, 1),
    ("Size", super::node_declaration::SOCK_FLOAT, 1),
    ("Streaks", super::node_declaration::SOCK_INT, 1),
    ("Streaks Angle", super::node_declaration::SOCK_FLOAT, 1),
    ("Iterations", super::node_declaration::SOCK_INT, 1),
    ("Fade", super::node_declaration::SOCK_FLOAT, 1),
    ("Color Modulation", super::node_declaration::SOCK_FLOAT, 1),
    ("Diagonal", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Sun Position", super::node_declaration::SOCK_VECTOR, 1),
    ("Jitter", super::node_declaration::SOCK_FLOAT, 1),
    ("Kernel Data Type", super::node_declaration::SOCK_MENU, 1),
    ("Kernel", super::node_declaration::SOCK_FLOAT, 1),
    ("Kernel", super::node_declaration::SOCK_RGBA, 1),
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
/// `update` (verbatim — socket-level update hook; per-node body pending).
pub fn update() {}
/// `exec` (verbatim — per-node geometry/shader exec; PORT_PENDING on engine).
pub fn exec() {}
