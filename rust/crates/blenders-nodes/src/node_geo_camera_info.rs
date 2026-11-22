/// Port of `geometry/nodes/node_geo_camera_info.cc` — node `node_geo_camera_info`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Projection Matrix", super::node_declaration::SOCK_MATRIX, 2),
    ("Focal Length", super::node_declaration::SOCK_FLOAT, 2),
    ("Sensor", super::node_declaration::SOCK_VECTOR, 2),
    ("Shift", super::node_declaration::SOCK_VECTOR, 2),
    ("Clip Start", super::node_declaration::SOCK_FLOAT, 2),
    ("Clip End", super::node_declaration::SOCK_FLOAT, 2),
    ("Focus Distance", super::node_declaration::SOCK_FLOAT, 2),
    ("Is Orthographic", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Orthographic Scale", super::node_declaration::SOCK_FLOAT, 2),
    ("Camera", super::node_declaration::SOCK_OBJECT, 1),
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
