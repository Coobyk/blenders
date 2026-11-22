/// Port of `geometry/nodes/node_geo_sample_sound_frequencies.cc` — node `node_geo_sample_sound_frequencies`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Amplitude", super::node_declaration::SOCK_FLOAT, 2),
    ("Sound", super::node_declaration::SOCK_FLOAT, 1),
    ("Time", super::node_declaration::SOCK_FLOAT, 1),
    ("All Channels", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Channel", super::node_declaration::SOCK_INT, 1),
    ("Low", super::node_declaration::SOCK_FLOAT, 1),
    ("High", super::node_declaration::SOCK_FLOAT, 1),
    ("FFT Size", super::node_declaration::SOCK_MENU, 1),
    ("Window Function", super::node_declaration::SOCK_MENU, 1),
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
