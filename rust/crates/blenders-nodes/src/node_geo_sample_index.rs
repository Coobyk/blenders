/// Port of `geometry/nodes/node_geo_sample_index.cc` — node `node_geo_sample_index`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Geometry", super::node_declaration::SOCK_GEOMETRY, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 1),
    ("Index", super::node_declaration::SOCK_INT, 1),
    ("Clamp", super::node_declaration::SOCK_BOOLEAN, 1),
    ("Value", super::node_declaration::SOCK_FLOAT, 2),
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
/// `exec` (verbatim — sample `values` at
/// `index`, wrap or clamp per `clamp` —
/// C clamps when use_clamp else wraps).
pub fn exec(values: &[f32], index: i64, clamp: bool) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let n = values.len() as i64;
    let i = if clamp {
        index.clamp(0, n - 1)
    } else {
        index.rem_euclid(n)
    };
    values[i as usize]
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
