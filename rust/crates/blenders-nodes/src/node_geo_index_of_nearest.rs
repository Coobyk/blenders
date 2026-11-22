/// Port of `geometry/nodes/node_geo_index_of_nearest.cc` — node `node_geo_index_of_nearest`
/// declaration + exec (verbatim sockets).
/// `SOCKS` (verbatim — declared
/// input/output sockets).
pub const SOCKS: &[(&str, i32, i32)] = &[
    ("Position", super::node_declaration::SOCK_VECTOR, 1),
    ("Group ID", super::node_declaration::SOCK_INT, 1),
    ("Index", super::node_declaration::SOCK_INT, 2),
    ("Has Neighbor", super::node_declaration::SOCK_BOOLEAN, 2),
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
/// `exec` (verbatim — brute-force nearest
/// point (excluding self); BVH → linear
/// scan PORT_NOTE).
pub fn exec(
    query: [f32; 3],
    positions: &[[f32; 3]],
    self_index: i64,
    group_id: i64,
    group_ids: &[i64],
) -> (i64, bool) {
    let mut best = -1i64;
    let mut best_d = f32::MAX;
    for (i, p) in positions.iter().enumerate() {
        if i as i64 == self_index {
            continue;
        }
        if !group_ids.is_empty() && group_ids[i] != group_id {
            continue;
        }
        let d = (p[0] - query[0]).powi(2)
            + (p[1] - query[1]).powi(2)
            + (p[2] - query[2]).powi(2);
        if d < best_d {
            best_d = d;
            best = i as i64;
        }
    }
    (best, best >= 0)
}
/// `update` (verbatim — socket-level update hook).
pub fn update() {}
