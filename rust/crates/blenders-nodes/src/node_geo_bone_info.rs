/// Port of `geometry/nodes/node_geo_bone_info.cc` — node `node_geo_bone_info`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Pose", super::node_declaration::SOCK_MATRIX, 2),
    ("Local Pose", super::node_declaration::SOCK_MATRIX, 2),
    ("Transform Pose", super::node_declaration::SOCK_MATRIX, 2),
    ("Rest Pose", super::node_declaration::SOCK_MATRIX, 2),
    ("Rest Length", super::node_declaration::SOCK_FLOAT, 2),
    ("Exists", super::node_declaration::SOCK_BOOLEAN, 2),
    ("Envelope Distance", super::node_declaration::SOCK_FLOAT, 2),
    ("Radius Head", super::node_declaration::SOCK_FLOAT, 2),
    ("Radius Tail", super::node_declaration::SOCK_FLOAT, 2),
    ("Armature", super::node_declaration::SOCK_OBJECT, 1),
    ("Bone Name", super::node_declaration::SOCK_STRING, 1),
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
